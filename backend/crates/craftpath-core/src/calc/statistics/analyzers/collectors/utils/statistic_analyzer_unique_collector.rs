use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use humansize::SizeFormatter;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};

use crate::calc::statistics::helpers::RouteCustomWeight;
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

#[derive(Clone)]
struct RankedRoute<'a> {
    inner: ItemRouteRef<'a>,
    lower_is_better: bool,
}

impl<'a> PartialEq for RankedRoute<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.weight == other.inner.weight
    }
}
impl<'a> Eq for RankedRoute<'a> {}

impl<'a> PartialOrd for RankedRoute<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for RankedRoute<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self
            .inner
            .weight
            .partial_cmp(&other.inner.weight)
            .unwrap_or(Ordering::Equal);

        match self.lower_is_better {
            true => ordering,
            false => ordering.reverse(),
        }
    }
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

    let mut stack: Vec<(Vec<ItemRouteNodeRef>, RouteCustomWeight, &ItemMatrixNode)> = Vec::new();

    let mut heap: BinaryHeap<RankedRoute> = BinaryHeap::new();
    let mut actual_ram: u64 = 0;

    let tree = &calculator.matrix;
    let start = calculator
        .matrix
        .get(&hash_value(&calculator.starting_item))
        .ok_or_else(|| anyhow!("Did not find starting item in the matrix."))?;

    // initialize stack with empty path and zero weight if lowIsB and 1 if not
    stack.push((
        Vec::new(),
        if lower_is_better {
            RouteCustomWeight::from(f64::NEG_INFINITY)
        } else {
            RouteCustomWeight::from(f64::INFINITY)
        },
        start,
    ));

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

    while let Some((path, acc_weight, node)) = stack.pop() {
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
            let speed = (count as f64 / elapsed).round() as u64;
            let accepted_routes = heap.len();
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
            ));
        }

        if node.item.helper.target_proximity == 0 {
            count_finished += 1;

            let weight = T::get_weight(&path, &calculator.matrix, item_provider, market_provider);

            let route = ItemRouteRef {
                route: path,
                weight: weight.0,
                chance: weight.1,
            };

            let ranked = RankedRoute {
                inner: route,
                lower_is_better,
            };

            let route_ram = ram_usage_item_route_ref(&ranked.inner);

            if heap.len() < max_routes as usize {
                actual_ram += route_ram;
                heap.push(ranked);
            } else {
                let worst = heap.peek().unwrap();
                let improves = if lower_is_better {
                    ranked.inner.weight < worst.inner.weight
                } else {
                    ranked.inner.weight > worst.inner.weight
                };
                if improves {
                    let removed = heap.pop().unwrap();
                    let removed_ram = ram_usage_item_route_ref(&removed.inner);
                    actual_ram = actual_ram.saturating_sub(removed_ram);

                    actual_ram += route_ram;
                    heap.push(ranked);
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

                    // prune based on accumulated weight
                    if heap.len() == max_routes as usize {
                        if let Some(worst) = heap.peek() {
                            let prune = if lower_is_better {
                                acc_weight > worst.inner.weight
                            } else {
                                acc_weight < worst.inner.weight
                            };
                            if prune {
                                continue;
                            }
                        }
                    }

                    let mut new_path = path.clone();
                    new_path.push(ItemRouteNodeRef {
                        item: &target.next,
                        chance: &target.chance,
                        currency_list,
                    });

                    // new accumulated weight
                    let accumulated_weight = T::get_weight(
                        &new_path,
                        &calculator.matrix,
                        &item_provider,
                        &market_provider,
                    );

                    stack.push((new_path, accumulated_weight.0, next_node));
                } else {
                    tracing::warn!("Missing node for {:?}", target.next);
                }
            }
        }
    }

    let results: Vec<ItemRouteRef> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|r| r.inner)
        .collect();

    Ok(results)
}
