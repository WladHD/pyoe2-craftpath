//! Dijkstra + Yen's K best loopless paths, generic over a monotone
//! [`EdgeMetric`], plus an exact branch-and-bound for the efficiency metric.
//!
//! Monotonicity (extending a path never improves the accumulator — chance
//! multiplies factors ≤ 1, cost adds ≥ 0) makes Dijkstra's settle-order
//! argument valid for both metrics, and guarantees Yen enumerates simple
//! paths in non-improving metric order.

use anyhow::Result;

use crate::{
    api::{
        calculator::Calculator,
        errors::CraftPathError,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, ItemRouteRef, StatisticAnalyzerCollectorTrait,
    },
    progress::ProgressSink,
};

use super::{
    graph::CraftGraph,
    metrics::{ChanceMetric, CostMetric, EdgeMetric, better},
};

/// A simple path = ordered list of global edge indices from the start node.
type EdgePath = Vec<u32>;

struct HeapEntry {
    accum: f64,
    node: u32,
}

/// Dijkstra from `source` to the nearest terminal under metric `M`,
/// honouring banned nodes/edges (Yen's spur machinery). Returns the edge
/// path and its accumulator, or None if no terminal is reachable.
fn dijkstra_to_terminal<M: EdgeMetric>(
    graph: &CraftGraph<'_>,
    source: u32,
    banned_nodes: &[bool],
    banned_edges: &[bool],
) -> Option<(EdgePath, f64)> {
    let n = graph.nodes.len();
    let mut best: Vec<f64> = vec![if M::LOWER_IS_BETTER { f64::INFINITY } else { -1.0 }; n];
    let mut settled: Vec<bool> = vec![false; n];
    let mut via_edge: Vec<u32> = vec![u32::MAX; n];
    let mut via_node: Vec<u32> = vec![u32::MAX; n];

    // simple binary heap on (accum, node); ordering flips with the metric
    let mut heap: Vec<HeapEntry> = Vec::new();
    let push = |heap: &mut Vec<HeapEntry>, entry: HeapEntry| {
        heap.push(entry);
        let mut i = heap.len() - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if better::<M>(heap[i].accum, heap[parent].accum) {
                heap.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    };
    let pop = |heap: &mut Vec<HeapEntry>| -> Option<HeapEntry> {
        if heap.is_empty() {
            return None;
        }
        let last = heap.len() - 1;
        heap.swap(0, last);
        let top = heap.pop().unwrap();
        let mut i = 0;
        loop {
            let (l, r) = (2 * i + 1, 2 * i + 2);
            let mut best_child = i;
            if l < heap.len() && better::<M>(heap[l].accum, heap[best_child].accum) {
                best_child = l;
            }
            if r < heap.len() && better::<M>(heap[r].accum, heap[best_child].accum) {
                best_child = r;
            }
            if best_child == i {
                break;
            }
            heap.swap(i, best_child);
            i = best_child;
        }
        Some(top)
    };

    best[source as usize] = M::init();
    push(&mut heap, HeapEntry { accum: M::init(), node: source });

    while let Some(entry) = pop(&mut heap) {
        let u = entry.node;
        if settled[u as usize] {
            continue;
        }
        settled[u as usize] = true;

        if graph.is_terminal(u) {
            // reconstruct
            let mut path: EdgePath = Vec::new();
            let mut cursor = u;
            while via_edge[cursor as usize] != u32::MAX {
                path.push(via_edge[cursor as usize]);
                cursor = via_node[cursor as usize];
            }
            path.reverse();
            return Some((path, entry.accum));
        }

        for (offset, edge) in graph.out_edges(u).iter().enumerate() {
            let edge_idx = graph.edge_index(u, offset) as u32;
            if banned_edges[edge_idx as usize] || banned_nodes[edge.to as usize] {
                continue;
            }
            let candidate = M::extend(best[u as usize], edge);
            if !settled[edge.to as usize] && better::<M>(candidate, best[edge.to as usize]) {
                best[edge.to as usize] = candidate;
                via_edge[edge.to as usize] = edge_idx;
                via_node[edge.to as usize] = u;
                push(&mut heap, HeapEntry { accum: candidate, node: edge.to });
            }
        }
    }

    None
}

fn accum_of<M: EdgeMetric>(graph: &CraftGraph<'_>, path: &EdgePath) -> f64 {
    path.iter()
        .fold(M::init(), |acc, &e| M::extend(acc, &graph.edges[e as usize]))
}

/// Yen's algorithm: lazily yields the K best loopless paths under `M` in
/// non-improving order. `visit` receives each path; returning `false` stops
/// the enumeration early (used by the efficiency bound).
fn yen_enumerate<M: EdgeMetric>(
    graph: &CraftGraph<'_>,
    k_max: usize,
    sink: &dyn ProgressSink,
    mut visit: impl FnMut(&EdgePath, f64) -> bool,
) -> Result<()> {
    let n = graph.nodes.len();
    let e = graph.edges.len();
    let mut banned_nodes = vec![false; n];
    let mut banned_edges = vec![false; e];

    // start node can itself be terminal: the empty path
    if graph.is_terminal(graph.start) {
        let _ = visit(&Vec::new(), M::init());
        return Ok(());
    }

    let Some(first) = dijkstra_to_terminal::<M>(graph, graph.start, &banned_nodes, &banned_edges)
    else {
        return Ok(());
    };

    let mut accepted: Vec<EdgePath> = vec![first.0.clone()];
    if !visit(&first.0, first.1) {
        return Ok(());
    }

    // candidate pool of (accum, path) not yet accepted
    let mut candidates: Vec<(f64, EdgePath)> = Vec::new();

    for round in 1..k_max {
        if sink.is_cancelled() {
            return Err(CraftPathError::Cancelled().into());
        }
        sink.report(
            &format!("Fast engine: {} routes accepted, {} candidates", accepted.len(), candidates.len()),
            accepted.len() as u64,
            Some(k_max as u64),
        );

        let previous = accepted[round - 1].clone();

        // spur from every prefix position of the previous path
        let mut spur_node = graph.start;
        for spur_pos in 0..=previous.len().saturating_sub(1) {
            let root: &[u32] = &previous[..spur_pos];

            // ban edges used by accepted paths sharing this root
            for path in &accepted {
                if path.len() > spur_pos && path[..spur_pos] == *root {
                    banned_edges[path[spur_pos] as usize] = true;
                }
            }
            // ban root nodes (looplessness)
            let mut cursor = graph.start;
            for &edge in root {
                banned_nodes[cursor as usize] = true;
                cursor = graph.edges[edge as usize].to;
            }
            spur_node = cursor;

            if let Some((spur_path, _)) =
                dijkstra_to_terminal::<M>(graph, spur_node, &banned_nodes, &banned_edges)
            {
                let mut full: EdgePath = root.to_vec();
                full.extend(spur_path);
                let accum = accum_of::<M>(graph, &full);
                if !accepted.contains(&full) && !candidates.iter().any(|(_, p)| p == &full) {
                    candidates.push((accum, full));
                }
            }

            // reset bans
            banned_nodes.iter_mut().for_each(|b| *b = false);
            banned_edges.iter_mut().for_each(|b| *b = false);
        }
        let _ = spur_node;

        // pick the best candidate
        let Some(best_idx) = candidates
            .iter()
            .enumerate()
            .max_by(|(_, (a, _)), (_, (b, _))| {
                if better::<M>(*a, *b) {
                    std::cmp::Ordering::Greater
                } else if better::<M>(*b, *a) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .map(|(i, _)| i)
        else {
            break; // exhausted
        };

        let (accum, path) = candidates.swap_remove(best_idx);
        accepted.push(path.clone());
        if !visit(&path, accum) {
            return Ok(());
        }
    }

    Ok(())
}

fn reconstruct<'a>(graph: &CraftGraph<'a>, path: &EdgePath) -> Vec<ItemRouteNodeRef<'a>> {
    path.iter()
        .map(|&e| {
            let edge = &graph.edges[e as usize];
            ItemRouteNodeRef {
                item: edge.next_snapshot,
                chance: edge.chance,
                currency_list: edge.currency_list,
            }
        })
        .collect()
}

fn check_ram(graph: &CraftGraph<'_>, max_ram_in_bytes: u64) -> Result<()> {
    // Yen's working set is O(V + E + K·L); the graph dominates.
    let estimate = graph.ram_estimate() * 2;
    if estimate > max_ram_in_bytes {
        return Err(CraftPathError::RamLimitReached(format!("{} bytes", max_ram_in_bytes)).into());
    }
    Ok(())
}

/// Exact top-K simple paths under a decomposable metric. Final
/// `weight`/`chance` recomputed via the legacy collector `T` for bit-parity.
pub fn k_best_paths<'a, M: EdgeMetric, T: StatisticAnalyzerCollectorTrait>(
    graph: &CraftGraph<'a>,
    calculator: &'a Calculator,
    item_provider: &ItemInfoProvider,
    market_provider: &MarketPriceProvider,
    k: usize,
    max_ram_in_bytes: u64,
    sink: &dyn ProgressSink,
) -> Result<Vec<ItemRouteRef<'a>>> {
    if k == 0 {
        return Ok(Vec::new());
    }
    check_ram(graph, max_ram_in_bytes)?;

    let mut results: Vec<ItemRouteRef<'a>> = Vec::new();
    yen_enumerate::<M>(graph, k, sink, |path, _| {
        let nodes = reconstruct(graph, path);
        let (weight, chance) =
            T::get_weight(&nodes, &calculator.matrix, item_provider, market_provider);
        results.push(ItemRouteRef {
            route: nodes,
            weight,
            chance,
        });
        true
    })?;

    Ok(results)
}

/// Identical formula to the legacy efficient-cost collector.
#[inline]
pub fn efficiency(cost: f64, chance: f64) -> f64 {
    let tries = ((((1.0_f64 - 0.6_f64).ln() / (1.0_f64 - chance).ln()).ceil()) as u64).max(1);
    cost * tries as f64
}

/// Top-K by efficiency (`cost × tries60(chance)` — not edge-decomposable)
/// via bi-criteria label-setting search:
///
/// - Labels `(cost, chance, parent)` are processed in ascending **cost**
///   order from a global queue. Edge costs are > 0 (zero prices are clamped
///   to an epsilon internally), so by the Dijkstra settle argument a popped
///   label's cost is final — label-setting, no correcting.
/// - Per node, a label is discarded iff ≥ K already-settled labels dominate
///   it (cost ≤ and chance ≥). Both accumulators are suffix-monotone, so
///   each dominator extended by the same suffix is at least as good — K
///   dominators ⇒ the label cannot reach the global top-K (exact).
/// - Cycles need no special handling: a path containing a cycle is strictly
///   cost-dominated by its simple reduction, so it is pruned by dominance.
/// - `frontier_cap` bounds per-node settled labels — the only inexact knob;
///   exact whenever the K-dominance frontier fits within it.
pub fn k_best_efficiency_paths<'a, T: StatisticAnalyzerCollectorTrait>(
    graph: &CraftGraph<'a>,
    calculator: &'a Calculator,
    item_provider: &ItemInfoProvider,
    market_provider: &MarketPriceProvider,
    k: usize,
    frontier_cap: usize,
    max_ram_in_bytes: u64,
    sink: &dyn ProgressSink,
) -> Result<Vec<ItemRouteRef<'a>>> {
    if k == 0 {
        return Ok(Vec::new());
    }
    check_ram(graph, max_ram_in_bytes)?;

    #[derive(Clone, Copy)]
    struct BiLabel {
        cost: f64,
        chance: f64,
        parent: u32,
        edge: u32,
    }
    const ROOT: u32 = u32::MAX;

    let n = graph.nodes.len();
    let mut arena: Vec<BiLabel> = Vec::new();
    let mut settled: Vec<Vec<u32>> = vec![Vec::new(); n];
    // node of each arena label (parallel array; avoids storing it per label)
    let mut label_node: Vec<u32> = Vec::new();

    // total-order wrapper for f64 costs (no NaNs are produced here)
    #[derive(PartialEq, PartialOrd)]
    struct OrderedF64(f64);
    impl Eq for OrderedF64 {}
    impl Ord for OrderedF64 {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
        }
    }

    // min-heap on cost: (cost, label_id)
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(OrderedF64, u32)>> =
        std::collections::BinaryHeap::new();

    arena.push(BiLabel { cost: 0.0, chance: 1.0, parent: ROOT, edge: 0 });
    label_node.push(graph.start);
    heap.push(std::cmp::Reverse((OrderedF64(0.0), 0)));

    let dominates = |a: &BiLabel, b: &BiLabel| -> bool {
        a.cost <= b.cost && a.chance >= b.chance && (a.cost < b.cost || a.chance > b.chance)
    };

    let mut processed: u64 = 0;
    while let Some(std::cmp::Reverse((_, label_id))) = heap.pop() {
        processed += 1;
        if processed % 4096 == 0 {
            if sink.is_cancelled() {
                return Err(CraftPathError::Cancelled().into());
            }
            let label_ram = (arena.capacity() * size_of::<BiLabel>()) as u64;
            if graph.ram_estimate() + label_ram > max_ram_in_bytes {
                return Err(
                    CraftPathError::RamLimitReached(format!("{} bytes", max_ram_in_bytes)).into(),
                );
            }
            sink.report(
                &format!(
                    "Fast engine (efficiency): {} labels processed, {} in arena",
                    processed,
                    arena.len()
                ),
                processed,
                None,
            );
        }

        let label = arena[label_id as usize];
        let node = label_node[label_id as usize];

        // K-dominance + capacity check against already-settled labels
        let bucket = &settled[node as usize];
        if bucket.len() >= frontier_cap {
            continue;
        }
        let dominators = bucket
            .iter()
            .filter(|id| dominates(&arena[**id as usize], &label))
            .count();
        if dominators >= k {
            continue;
        }
        settled[node as usize].push(label_id);

        if graph.is_terminal(node) {
            continue; // terminals collect below; no expansion past them
        }

        for (offset, edge) in graph.out_edges(node).iter().enumerate() {
            let edge_idx = graph.edge_index(node, offset) as u32;
            // clamp: the label-setting argument needs strictly positive cost
            let edge_cost = edge.cost_ex.max(f64::MIN_POSITIVE);
            let candidate = BiLabel {
                cost: label.cost + edge_cost,
                chance: label.chance * edge.chance_f64,
                parent: label_id,
                edge: edge_idx,
            };
            let id = arena.len() as u32;
            arena.push(candidate);
            label_node.push(edge.to);
            heap.push(std::cmp::Reverse((OrderedF64(candidate.cost), id)));
        }
    }

    // top-K terminal labels by final efficiency
    let mut terminal: Vec<(f64, u32)> = Vec::new();
    for &node in &graph.terminals {
        for &id in &settled[node as usize] {
            let label = arena[id as usize];
            terminal.push((efficiency(label.cost, label.chance), id));
        }
    }
    terminal.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    terminal.truncate(k);

    let mut results: Vec<ItemRouteRef<'a>> = Vec::with_capacity(terminal.len());
    for (_, id) in terminal {
        let mut edge_path: EdgePath = Vec::new();
        let mut cursor = id;
        while arena[cursor as usize].parent != ROOT {
            edge_path.push(arena[cursor as usize].edge);
            cursor = arena[cursor as usize].parent;
        }
        edge_path.reverse();
        let nodes = reconstruct(graph, &edge_path);
        let (weight, chance) =
            T::get_weight(&nodes, &calculator.matrix, item_provider, market_provider);
        results.push(ItemRouteRef {
            route: nodes,
            weight,
            chance,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::super::graph::{CraftGraph, test_support::*};
    use super::*;
    use crate::api::currency::CraftCurrencyEnum;
    use crate::calc::statistics::analyzers::collectors::unique_paths::chance_collector::UniquePathChanceCollector;
    use crate::progress::NoopProgress;
    use crate::utils::fraction_utils::Fraction;

    /// diamond: s -> a (1/2) -> t (1/3) = 1/6 ; s -> b (1/4) -> t (1/5) = 1/20
    fn diamond() -> crate::api::calculator::Calculator {
        let s = snapshot(10);
        let a = snapshot(20);
        let b = snapshot(30);
        let t = snapshot(40);

        let mut ns = node(s.clone(), 2);
        let mut na = node(a.clone(), 1);
        let mut nb = node(b.clone(), 1);
        let nt = node(t.clone(), 0);

        link(&mut ns, currency(CraftCurrencyEnum::ChaosOrbNormal()), &a, Fraction::new(1, 2));
        link(&mut ns, currency(CraftCurrencyEnum::ExaltedOrbNormal()), &b, Fraction::new(1, 4));
        link(&mut na, currency(CraftCurrencyEnum::RegalOrbNormal()), &t, Fraction::new(1, 3));
        link(&mut nb, currency(CraftCurrencyEnum::VaalOrb()), &t, Fraction::new(1, 5));

        calculator(s, t, vec![ns, na, nb, nt])
    }

    #[test]
    fn test_yen_chance_on_diamond() {
        let calc = diamond();
        let (ip, mp) = empty_providers();
        let graph = CraftGraph::build(&calc, &ip, &mp).unwrap();

        let routes = k_best_paths::<ChanceMetric, UniquePathChanceCollector>(
            &graph, &calc, &ip, &mp, 5, 1_000_000_000, &NoopProgress,
        )
        .unwrap();

        assert_eq!(routes.len(), 2);
        assert!((routes[0].chance.get_raw_value() - 1.0 / 6.0).abs() < 1e-12);
        assert!((routes[1].chance.get_raw_value() - 1.0 / 20.0).abs() < 1e-12);
        assert_eq!(routes[0].route.len(), 2);
    }

    #[test]
    fn test_yen_handles_cycles_loopless() {
        // s <-> a cycle with exit to t: only 2 simple paths exist
        // s -> t (1/10) and s -> a -> ... a cannot return to s in a simple
        // path, a -> t (1/2 * 1/3)
        let s = snapshot(10);
        let a = snapshot(20);
        let t = snapshot(30);

        let mut ns = node(s.clone(), 1);
        let mut na = node(a.clone(), 1);
        let nt = node(t.clone(), 0);
        link(&mut ns, currency(CraftCurrencyEnum::ChaosOrbNormal()), &a, Fraction::new(1, 2));
        link(&mut na, currency(CraftCurrencyEnum::RegalOrbNormal()), &s, Fraction::new(9, 10));
        link(&mut na, currency(CraftCurrencyEnum::VaalOrb()), &t, Fraction::new(1, 3));
        link(&mut ns, currency(CraftCurrencyEnum::ExaltedOrbNormal()), &t, Fraction::new(1, 10));

        let calc = calculator(s, t, vec![ns, na, nt]);
        let (ip, mp) = empty_providers();
        let graph = CraftGraph::build(&calc, &ip, &mp).unwrap();

        let routes = k_best_paths::<ChanceMetric, UniquePathChanceCollector>(
            &graph, &calc, &ip, &mp, 10, 1_000_000_000, &NoopProgress,
        )
        .unwrap();

        assert_eq!(routes.len(), 2, "exactly two simple paths");
        assert!((routes[0].chance.get_raw_value() - 1.0 / 6.0).abs() < 1e-12);
        assert!((routes[1].chance.get_raw_value() - 1.0 / 10.0).abs() < 1e-12);
    }

    #[test]
    fn test_start_is_terminal_yields_empty_route() {
        let s = snapshot(10);
        let ns = node(s.clone(), 0);
        let calc = calculator(s.clone(), s.clone(), vec![ns]);
        let (ip, mp) = empty_providers();
        let graph = CraftGraph::build(&calc, &ip, &mp).unwrap();

        let routes = k_best_paths::<ChanceMetric, UniquePathChanceCollector>(
            &graph, &calc, &ip, &mp, 5, 1_000_000_000, &NoopProgress,
        )
        .unwrap();
        assert_eq!(routes.len(), 1);
        assert!(routes[0].route.is_empty());
    }

    #[test]
    fn test_efficiency_branch_and_bound() {
        // cheap-unlikely vs expensive-likely tradeoff (see pareto test of old
        // engine): exalt route eff 8 must beat chaos route eff 18.4
        use crate::api::provider::market_prices::{ItemName, PriceInDivines};
        use crate::api::types::THashMap;
        use crate::calc::statistics::analyzers::collectors::unique_paths::efficient_cost_collector::UniquePathEfficientCostCollector;

        let s = snapshot(10);
        let a = snapshot(20);
        let b = snapshot(30);
        let t = snapshot(40);

        let mut ns = node(s.clone(), 2);
        let mut na = node(a.clone(), 1);
        let mut nb = node(b.clone(), 1);
        let nt = node(t.clone(), 0);

        link(&mut ns, currency(CraftCurrencyEnum::ChaosOrbNormal()), &a, Fraction::new(1, 10));
        link(&mut na, currency(CraftCurrencyEnum::ChaosOrbNormal()), &t, Fraction::new(1, 10));
        link(&mut ns, currency(CraftCurrencyEnum::ExaltedOrbNormal()), &b, Fraction::new(1, 2));
        link(&mut nb, currency(CraftCurrencyEnum::ExaltedOrbNormal()), &t, Fraction::new(1, 2));

        let calc = calculator(s, t, vec![ns, na, nb, nt]);

        let item_provider = crate::api::provider::item_info::ItemInfoProvider {
            cache_affix_def: THashMap::default(),
            cache_item_affix_table: THashMap::default(),
            cache_affix_essence_table: THashMap::default(),
            cache_essence_def: THashMap::default(),
            cache_base_group_table: THashMap::default(),
            base_group_definition: THashMap::default(),
        };
        let mut prices = THashMap::default();
        prices.insert(ItemName::from("Chaos Orb".to_string()), PriceInDivines::new(0.1));
        prices.insert(ItemName::from("Exalted Orb".to_string()), PriceInDivines::new(1.0));
        let market = crate::api::provider::market_prices::MarketPriceProvider {
            cache_market_prices: prices,
            cache_exchange_rate_div_to_exalted: 1.0,
            cache_exchange_rate_div_to_chaos: 10.0,
        };

        let graph = CraftGraph::build(&calc, &item_provider, &market).unwrap();
        let routes = k_best_efficiency_paths::<UniquePathEfficientCostCollector>(
            &graph, &calc, &item_provider, &market, 5, 256, 1_000_000_000, &NoopProgress,
        )
        .unwrap();

        assert_eq!(routes.len(), 2);
        let first = routes[0].weight.get_raw_value();
        assert!((first - 8.0).abs() < 1e-9, "exalt route weight: {first}");
        assert!(routes[0].weight.get_raw_value() < routes[1].weight.get_raw_value());
    }
}
