//! CSR snapshot of the item matrix with per-edge data resolved once.

use anyhow::{Result, anyhow};

use crate::{
    api::{
        calculator::{Calculator, ItemMatrixNode},
        currency::CraftCurrencyList,
        item::ItemSnapshot,
        provider::{
            item_info::ItemInfoProvider,
            market_prices::{MarketPriceProvider, PriceKind},
        },
        types::THashMap,
    },
    utils::{fraction_utils::Fraction, hash_utils::hash_value},
};

/// One outgoing edge with everything the analyzers need, resolved once.
pub struct GraphEdge<'a> {
    pub to: u32,
    /// `target.chance.to_f64()` - the hot-loop accumulator input.
    pub chance_f64: f64,
    /// Exact fraction for route reconstruction.
    pub chance: &'a Fraction,
    /// Price of the edge's currency list in Exalted Orbs, resolved once
    /// (identical expression to the legacy cost collectors).
    pub cost_ex: f64,
    pub currency_list: &'a CraftCurrencyList,
    /// The snapshot the edge leads to (legacy `ItemRouteNodeRef.item`).
    pub next_snapshot: &'a ItemSnapshot,
}

pub struct CraftGraph<'a> {
    pub nodes: Vec<&'a ItemMatrixNode>,
    /// CSR offsets, len = nodes.len() + 1.
    edge_offsets: Vec<u32>,
    pub edges: Vec<GraphEdge<'a>>,
    /// Nodes with target_proximity == 0.
    pub terminals: Vec<u32>,
    pub start: u32,
}

impl<'a> CraftGraph<'a> {
    /// O(V + E) plus one price resolution per distinct currency list.
    pub fn build(
        calculator: &'a Calculator,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
    ) -> Result<Self> {
        let matrix = &calculator.matrix;

        // ---- node indexing (deterministic: sort hash keys) ----
        let mut keys: Vec<u64> = matrix.keys().cloned().collect();
        keys.sort_unstable();

        let mut nodes: Vec<&ItemMatrixNode> = Vec::with_capacity(keys.len());
        let mut node_index: THashMap<u64, u32> = THashMap::default();
        for key in &keys {
            node_index.insert(*key, nodes.len() as u32);
            nodes.push(matrix.get(key).unwrap());
        }

        let start_hash = hash_value(&calculator.starting_item);
        let start = *node_index
            .get(&start_hash)
            .ok_or_else(|| anyhow!("Did not find starting item in the matrix."))?;

        // ---- price memo per distinct currency list ----
        let mut price_memo: THashMap<&CraftCurrencyList, f64> = THashMap::default();
        let mut price_of = |list: &'a CraftCurrencyList| -> f64 {
            *price_memo.entry(list).or_insert_with(|| {
                list.list.iter().fold(0_f64, |a, b| {
                    a + market_provider.currency_convert(
                        &market_provider.try_lookup_currency_in_divines_default_if_fail(
                            b,
                            item_provider,
                        ),
                        &PriceKind::Exalted,
                    )
                })
            })
        };

        // ---- edges (CSR), validating strict proximity layering ----
        let mut edge_offsets: Vec<u32> = Vec::with_capacity(nodes.len() + 1);
        let mut edges: Vec<GraphEdge<'a>> = Vec::new();
        edge_offsets.push(0);

        for (from_idx, node) in nodes.iter().enumerate() {
            let mut node_edges: Vec<GraphEdge<'a>> = Vec::new();

            for (currency_list, targets) in node.propagate.iter() {
                for target in targets {
                    let Some(&to) = node_index.get(&hash_value(&target.next)) else {
                        tracing::warn!("Missing node for {:?}", target.next);
                        continue;
                    };

                    node_edges.push(GraphEdge {
                        to,
                        chance_f64: target.chance.to_f64(),
                        chance: &target.chance,
                        cost_ex: price_of(currency_list),
                        currency_list,
                        next_snapshot: &target.next,
                    });
                }
            }

            // deterministic edge order -> reproducible tie resolution
            node_edges.sort_unstable_by(|a, b| {
                a.cost_ex
                    .partial_cmp(&b.cost_ex)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(
                        b.chance_f64
                            .partial_cmp(&a.chance_f64)
                            .unwrap_or(std::cmp::Ordering::Equal),
                    )
                    .then(a.to.cmp(&b.to))
            });

            edges.extend(node_edges);
            edge_offsets.push(edges.len() as u32);
            let _ = from_idx;
        }

        let terminals: Vec<u32> = (0..nodes.len() as u32)
            .filter(|&i| nodes[i as usize].item.helper.target_proximity == 0)
            .collect();

        Ok(Self {
            nodes,
            edge_offsets,
            edges,
            terminals,
            start,
        })
    }

    #[inline]
    pub fn out_edges(&self, node: u32) -> &[GraphEdge<'a>] {
        let lo = self.edge_offsets[node as usize] as usize;
        let hi = self.edge_offsets[node as usize + 1] as usize;
        &self.edges[lo..hi]
    }

    /// Global index of a node's `offset`-th outgoing edge.
    #[inline]
    pub fn edge_index(&self, node: u32, offset: usize) -> usize {
        self.edge_offsets[node as usize] as usize + offset
    }

    #[inline]
    pub fn is_terminal(&self, node: u32) -> bool {
        self.nodes[node as usize].item.helper.target_proximity == 0
    }

    /// size_of-based estimate charged against `max_ram_in_bytes`.
    pub fn ram_estimate(&self) -> u64 {
        (self.nodes.len() * size_of::<&ItemMatrixNode>()
            + self.edge_offsets.len() * size_of::<u32>()
            + self.edges.len() * size_of::<GraphEdge>()
            + self.terminals.len() * size_of::<u32>()) as u64
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Hand-built tiny matrices for engine unit tests.

    use crate::api::calculator::{Calculator, ItemMatrixNode, PropagationTarget};
    use crate::domain::currency::{CraftCurrencyEnum, CraftCurrencyList};
    use crate::domain::item::{Item, ItemSnapshot, ItemSnapshotHelper, ItemTechnicalMeta};
    use crate::domain::types::{BaseItemId, ItemLevel, ItemRarityEnum, THashMap, THashSet};
    use crate::domain::fraction::Fraction;
    use crate::utils::hash_utils::hash_value;

    /// Distinct snapshots are produced by varying the item level.
    pub fn snapshot(level: u8) -> ItemSnapshot {
        ItemSnapshot {
            item_level: ItemLevel::from(level),
            rarity: ItemRarityEnum::Rare,
            base_id: BaseItemId::from(1),
            affixes: THashSet::default(),
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        }
    }

    pub fn node(snapshot: ItemSnapshot, proximity: u8) -> ItemMatrixNode {
        ItemMatrixNode {
            item: Item {
                snapshot,
                helper: ItemSnapshotHelper {
                    target_proximity: proximity,
                    prefix_count: 0,
                    suffix_count: 0,
                    blocked_modgroups: THashSet::default(),
                    homogenized_mods: THashSet::default(),
                    unwanted_affixes: THashSet::default(),
                    is_desecrated: false,
                    has_desecrated_target: None,
                    marked_by_abyssal_lord: None,
                    has_essences_target: THashSet::default(),
                },
                meta: ItemTechnicalMeta::default(),
            },
            propagate: THashMap::default(),
        }
    }

    pub fn currency(c: CraftCurrencyEnum) -> CraftCurrencyList {
        let mut list = CraftCurrencyList {
            list: THashSet::default(),
        };
        list.list.insert(c);
        list
    }

    pub fn link(
        from: &mut ItemMatrixNode,
        currency_list: CraftCurrencyList,
        to_snapshot: &ItemSnapshot,
        chance: Fraction,
    ) {
        from.propagate
            .entry(currency_list)
            .or_default()
            .push(PropagationTarget::new(chance, to_snapshot.clone()));
    }

    pub fn empty_providers() -> (
        crate::domain::provider::item_info::ItemInfoProvider,
        crate::domain::provider::market_prices::MarketPriceProvider,
    ) {
        (
            crate::domain::provider::item_info::ItemInfoProvider {
                cache_affix_def: THashMap::default(),
                cache_item_affix_table: THashMap::default(),
                cache_affix_essence_table: THashMap::default(),
                cache_essence_def: THashMap::default(),
                cache_base_group_table: THashMap::default(),
                base_group_definition: THashMap::default(),
            },
            crate::domain::provider::market_prices::MarketPriceProvider {
                cache_market_prices: THashMap::default(),
                cache_exchange_rate_div_to_exalted: 100.0,
                cache_exchange_rate_div_to_chaos: 1000.0,
            },
        )
    }

    pub fn calculator(start: ItemSnapshot, target: ItemSnapshot, nodes: Vec<ItemMatrixNode>) -> Calculator {
        let mut matrix = THashMap::default();
        for node in nodes {
            matrix.insert(hash_value(&node.item.snapshot), node);
        }
        Calculator {
            matrix,
            starting_item: start,
            target_item: target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::domain::currency::CraftCurrencyEnum;

    #[test]
    fn test_csr_shape_and_terminals() {
        // diamond: s(2) -> a(1), b(1) -> t(0)
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

        let calc = calculator(s.clone(), t.clone(), vec![ns, na, nb, nt]);
        let (item_provider, market) = empty_providers();
        let graph = CraftGraph::build(&calc, &item_provider, &market).unwrap();

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 4);
        assert_eq!(graph.out_edges(graph.start).len(), 2);
        assert_eq!(graph.terminals.len(), 1);
    }

    #[test]
    fn test_cyclic_matrix_builds_fine() {
        // a <-> b cycle plus an exit - Yen handles cycles, no error expected
        let s = snapshot(10);
        let a = snapshot(20);
        let t = snapshot(30);

        let mut ns = node(s.clone(), 1);
        let mut na = node(a.clone(), 1);
        let nt = node(t.clone(), 0);
        link(&mut ns, currency(CraftCurrencyEnum::ChaosOrbNormal()), &a, Fraction::new(1, 2));
        link(&mut na, currency(CraftCurrencyEnum::RegalOrbNormal()), &s, Fraction::new(1, 3));
        link(&mut na, currency(CraftCurrencyEnum::VaalOrb()), &t, Fraction::new(1, 5));

        let calc = calculator(s.clone(), t.clone(), vec![ns, na, nt]);
        let (item_provider, market) = empty_providers();
        let graph = CraftGraph::build(&calc, &item_provider, &market).unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.terminals.len(), 1);
    }
}
