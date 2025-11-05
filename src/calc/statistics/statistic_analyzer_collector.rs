use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use humansize::SizeFormatter;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};

use crate::{
    api::{
        calculator::{Calculator, ItemMatrix, ItemMatrixNode, ItemRoute, ItemRouteNode},
        currency::CraftCurrencyList,
        errors::CraftPathError,
        item::ItemSnapshot,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    utils::{fraction_utils::Fraction, hash_utils::hash_value},
};

pub trait StatisticAnalyzerCollectorTrait {
    fn get_weight(
        path: &Vec<ItemRouteNodeRef<'_>>,
        matrix: &ItemMatrix,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
    ) -> f64;
}

pub fn calculate_crafting_paths<'a, T: StatisticAnalyzerCollectorTrait>(
    calculator: &'a Calculator,
    item_provider: &'a ItemInfoProvider,
    market_provider: &'a MarketPriceProvider,
    max_routes: u32,
    max_ram_in_bytes: u64,
    lower_is_better: bool,
) -> Result<Vec<ItemRouteRef<'a>>> {
    tracing::info!("Generating unique craft paths based on item matrix");

    // current path, build for item
    let mut stack: Vec<(Vec<ItemRouteNodeRef>, &ItemMatrixNode)> = Vec::new();
    // sorted collection
    let mut results: Vec<ItemRouteRef> = Vec::new();

    let mut actual_ram: u64 = 0;

    let tree = &calculator.matrix;
    let start = calculator
        .matrix
        .get(&hash_value(&calculator.starting_item))
        .ok_or_else(|| anyhow!("Did not find starting item in the matrix."))?;

    stack.push((Vec::new(), start));

    let max_ram_show = SizeFormatter::new(max_ram_in_bytes, humansize::DECIMAL);

    let start_time = Instant::now();
    let mut count = 0usize;
    let mut count_finished = 0usize;
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠚⠞⠖⠦⠴⠲⠳⠓ "),
    );
    pb.enable_steady_tick(Duration::from_millis(500));

    while let Some((path, node)) = stack.pop() {
        count += 1;

        if count % 200_000 == 0 {
            if max_ram_in_bytes < actual_ram {
                return Err(CraftPathError::RamLimitReached(format!(
                    "{}",
                    SizeFormatter::new(max_ram_in_bytes, humansize::DECIMAL)
                ))
                .into());
            }

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = (count as f64 / elapsed).round() as u64; // integer paths/sec
            let accepted_routes = results.len();

            let est_ram_usage = SizeFormatter::new(actual_ram, humansize::DECIMAL);

            pb.set_message(format!(
                    "Applied {} currencies, resulting in {}/{} best, sorted routes (from a total of {}) [Speed: {} currencies/sec, RAM usage: {}/{}]",
                    count.to_formatted_string(&Locale::en),
                    accepted_routes.to_formatted_string(&Locale::en),
                    max_routes.to_formatted_string(&Locale::en),
                    count_finished.to_formatted_string(&Locale::en),
                    speed.to_formatted_string(&Locale::en),
                    est_ram_usage,
                    max_ram_show
                )
            );
        }

        if node.item.helper.target_proximity == 0 {
            count_finished += 1;
            // weight is gonna be calculated by statistic
            let weight = T::get_weight(&path, &calculator.matrix, &item_provider, &market_provider);

            let route = ItemRouteRef {
                route: path,
                weight,
            };

            if results.len() < max_routes as usize {
                // Insert sorted directly
                let pos = results
                    .binary_search_by(|r| {
                        if lower_is_better {
                            // smaller is better, ascending order
                            r.weight
                                .partial_cmp(&route.weight)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            // larger is better, descending order
                            route
                                .weight
                                .partial_cmp(&r.weight)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        }
                    })
                    .unwrap_or_else(|e| e);

                let accepted_route_ram = ram_usage_item_route_ref(&route);
                actual_ram += accepted_route_ram;

                results.insert(pos, route);
            } else {
                // Only insert if this route improves the worst one
                let worst = &results[results.len() - 1];
                let improves = if lower_is_better {
                    route.weight < worst.weight
                } else {
                    route.weight > worst.weight
                };

                if improves {
                    let pos = results
                        .binary_search_by(|r| {
                            if lower_is_better {
                                r.weight
                                    .partial_cmp(&route.weight)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                route
                                    .weight
                                    .partial_cmp(&r.weight)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            }
                        })
                        .unwrap_or_else(|e| e);

                    let accepted_route_ram = ram_usage_item_route_ref(&route);
                    actual_ram += accepted_route_ram;
                    let dropped_route_ram = ram_usage_item_route_ref(&results.last().unwrap());
                    actual_ram = actual_ram.saturating_sub(dropped_route_ram);

                    results.insert(pos, route);
                    results.pop(); // drop worst
                }
            }
            continue;
        }

        for (currency_list, targets) in &node.propagate {
            for target in targets {
                if let Some(next_node) = tree.get(&hash_value(&target.next)) {
                    // filter out cycles
                    if path
                        .iter()
                        .any(|test| test.item == &next_node.item.snapshot)
                    {
                        continue;
                    }

                    let mut new_path = path.clone();
                    new_path.push(ItemRouteNodeRef {
                        item: &node.item.snapshot,
                        chance: &target.chance,
                        currency_list: &currency_list,
                    });
                    stack.push((new_path, next_node));
                } else {
                    tracing::warn!("Missing node for {:?}", target.next);
                }
            }
        }
    }

    Ok(results)
}

#[derive(Clone, Debug)]
pub struct ItemRouteNodeRef<'a> {
    pub item: &'a ItemSnapshot,
    pub chance: &'a Fraction,
    pub currency_list: &'a CraftCurrencyList,
}

#[derive(Clone, Debug)]
pub struct ItemRouteRef<'a> {
    pub route: Vec<ItemRouteNodeRef<'a>>,
    pub weight: f64,
}

pub fn finalize_routes(mut routes: Vec<ItemRouteRef<'_>>) -> Vec<ItemRoute> {
    let mut finalized = Vec::with_capacity(routes.len());

    for route_ref in routes.drain(..) {
        let mut owned_nodes = Vec::with_capacity(route_ref.route.len());
        for node_ref in route_ref.route {
            owned_nodes.push(ItemRouteNode {
                item_matrix_id: hash_value(node_ref.item),
                chance: node_ref.chance.clone(),
                currency_list: node_ref.currency_list.clone(),
            });
        }
        finalized.push(ItemRoute {
            route: owned_nodes,
            weight: route_ref.weight,
        });
    }

    finalized
}

use std::mem::size_of;

/// Calculates RAM usage in bytes for an ItemRouteNodeRef<'a>.
pub fn ram_usage_item_route_node_ref<'a>(_node: &ItemRouteNodeRef<'a>) -> u64 {
    (size_of::<&ItemSnapshot>() + size_of::<&Fraction>() + size_of::<&CraftCurrencyList>()) as u64
}

/// Calculates RAM usage for an ItemRouteRef<'a> (includes Vec capacity).
pub fn ram_usage_item_route_ref<'a>(route_ref: &ItemRouteRef<'a>) -> u64 {
    let vec_capacity = route_ref.route.capacity();
    let node_size = size_of::<ItemRouteNodeRef<'a>>();
    let vec_overhead = size_of::<Vec<ItemRouteNodeRef<'a>>>();

    // route Vec + its allocated elements + f64 weight
    (vec_overhead + node_size * vec_capacity + size_of::<f64>()) as u64
}

/// Calculates RAM usage for a stack entry: (Vec<ItemRouteNodeRef>, &ItemMatrixNode)
pub fn ram_usage_stack_entry<'a>(path: &Vec<ItemRouteNodeRef<'a>>, _node: &ItemMatrixNode) -> u64 {
    let vec_capacity = path.capacity();
    let node_size = size_of::<ItemRouteNodeRef<'a>>();
    let vec_overhead = size_of::<Vec<ItemRouteNodeRef<'a>>>();
    let node_ref_size = size_of::<&ItemMatrixNode>();

    (vec_overhead + node_size * vec_capacity + node_ref_size) as u64
}
