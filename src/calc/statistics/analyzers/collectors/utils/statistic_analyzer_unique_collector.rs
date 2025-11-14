use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use humansize::SizeFormatter;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};

use crate::{
    api::{
        calculator::{Calculator, ItemMatrixNode},
        errors::CraftPathError,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, ItemRouteRef, StatisticAnalyzerCollectorTrait, ram_usage_item_route_ref,
    },
    utils::hash_utils::hash_value,
};

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
                weight: weight.0,
                chance: weight.1,
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
                        item: &target.next,
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
