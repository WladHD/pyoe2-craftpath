use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use humansize::SizeFormatter;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};

use crate::{
    api::{
        calculator::{Calculator, ItemMatrixNode},
        currency::CraftCurrencyList,
        errors::CraftPathError,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashMap,
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, RouteChance, StatisticAnalyzerCurrencyGroupCollectorTrait,
    },
    utils::hash_utils::hash_value,
};

pub fn calculate_currency_groups<'a, T: StatisticAnalyzerCurrencyGroupCollectorTrait>(
    calculator: &'a Calculator,
    item_provider: &'a ItemInfoProvider,
    market_provider: &'a MarketPriceProvider,
    max_ram_in_bytes: u64,
) -> Result<THashMap<Vec<&'a CraftCurrencyList>, Vec<Vec<RouteChance>>>> {
    tracing::info!("Generating unique craft paths based on item matrix");

    // current path, build for item
    let mut stack: Vec<(Vec<ItemRouteNodeRef>, &ItemMatrixNode)> = Vec::new();
    // sorted collection
    let mut results: THashMap<Vec<&CraftCurrencyList>, Vec<Vec<RouteChance>>> = THashMap::default();

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
    let mut collected = 0usize;
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
                    "Applied {} currencies, resulting in {} groups (from total of {} paths) [Speed: {} currencies/sec, RAM usage: {}/{}]",
                    count.to_formatted_string(&Locale::en),
                    accepted_routes.to_formatted_string(&Locale::en),
                    collected.to_formatted_string(&Locale::en),
                    speed.to_formatted_string(&Locale::en),
                    est_ram_usage,
                    max_ram_show
                )
            );
        }

        if node.item.helper.target_proximity == 0 {
            // weight is gonna be calculated by statistic
            collected += 1;
            let weights: Vec<RouteChance> =
                T::get_partial_weights(&path, &calculator.matrix, &item_provider, &market_provider);

            let path: Vec<&CraftCurrencyList> = path.iter().fold(Vec::new(), |mut a, b| {
                a.push(b.currency_list);
                a
            });
            // vec header + vec containting 1 chance
            actual_ram += 24 + (path.len() as u64) * (24 + 8);

            results.entry(path).or_default().push(weights);
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
