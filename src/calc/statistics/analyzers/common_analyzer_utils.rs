use anyhow::Result;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, GroupRoute},
        currency::CraftCurrencyList,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashMap,
    },
    calc::statistics::{
        analyzers::collectors::{
            currency_groups::group_chance_memory_efficient_collector::CurrencyGroupChanceMemoryEfficientCollector,
            utils::{
                statistic_analyzer_currency_grouped_collector::calculate_currency_groups,
                statistic_analyzer_currency_grouped_memory_efficient_collector::calculate_currency_groups_memory_efficient,
            },
        },
        helpers::{
            RouteChance, RouteCustomWeight, StatisticAnalyzerCurrencyGroupCollectorTrait,
            SubpathAmount,
        },
    },
    utils::float_compare,
};

#[instrument(skip_all)]
pub fn get_grouped_statistic_memory_efficient<T: StatisticAnalyzerCurrencyGroupCollectorTrait>(
    lower_is_better: bool,
    calculator: &Calculator,
    item_provider: &ItemInfoProvider,
    market_provider: &MarketPriceProvider,
    max_ram_in_bytes: u64,
) -> Result<Vec<GroupRoute>> {
    let res: THashMap<
        Vec<&CraftCurrencyList>,
        (
            RouteChance,
            RouteCustomWeight,
            SubpathAmount,
            Vec<RouteChance>,
        ),
    > = calculate_currency_groups_memory_efficient::<CurrencyGroupChanceMemoryEfficientCollector>(
        calculator,
        item_provider,
        market_provider,
        max_ram_in_bytes,
    )?;

    let mut data: Vec<GroupRoute> = res
        .into_par_iter()
        .map(|(k, v)| {
            let key_owned: Vec<CraftCurrencyList> = k.into_iter().cloned().collect();
            let chance: RouteChance = v.0;
            let weight: RouteCustomWeight = v.1;
            let subpaths_amount: SubpathAmount = v.2;
            let subpaths: Vec<RouteChance> = v.3;

            GroupRoute {
                group: key_owned,
                weight: weight,
                amount_subpaths: subpaths_amount,
                unique_route_weights: vec![subpaths],
                chance,
            }
        })
        .collect();

    if lower_is_better {
        data.par_sort_unstable_by(|a, b| {
            float_compare::cmp_f64(*a.weight.get_raw_value(), *b.weight.get_raw_value()).then(
                float_compare::cmp_f64(*a.chance.get_raw_value(), *b.chance.get_raw_value()),
            )
        });
    } else {
        data.par_sort_unstable_by(|a, b| {
            float_compare::cmp_f64(*b.weight.get_raw_value(), *a.weight.get_raw_value()).then(
                float_compare::cmp_f64(*a.chance.get_raw_value(), *b.chance.get_raw_value()),
            )
        });
    }

    Ok(data)
}

#[instrument(skip_all)]
pub fn get_grouped_statistic<T: StatisticAnalyzerCurrencyGroupCollectorTrait>(
    lower_is_better: bool,
    calculator: &Calculator,
    item_provider: &ItemInfoProvider,
    market_provider: &MarketPriceProvider,
    max_ram_in_bytes: u64,
) -> Result<Vec<GroupRoute>> {
    let res: THashMap<Vec<&CraftCurrencyList>, Vec<Vec<RouteChance>>> =
        calculate_currency_groups::<T>(
            calculator,
            item_provider,
            market_provider,
            max_ram_in_bytes,
        )?;

    let mut data: Vec<GroupRoute> = res
        .into_par_iter()
        .map(|(k, v)| {
            let chance = T::calculate_group_chance(&v);
            let weight = T::calculate_group_weight(&k, &v);

            GroupRoute {
                group: k.into_iter().cloned().collect(),
                weight,
                amount_subpaths: SubpathAmount::from(v.len() as u32),
                unique_route_weights: v,
                chance,
            }
        })
        .collect();

    if lower_is_better {
        data.par_sort_unstable_by(|a, b| {
            float_compare::cmp_f64(*a.weight.get_raw_value(), *b.weight.get_raw_value()).then(
                float_compare::cmp_f64(*a.chance.get_raw_value(), *b.chance.get_raw_value()),
            )
        });
    } else {
        data.par_sort_unstable_by(|a, b| {
            float_compare::cmp_f64(*b.weight.get_raw_value(), *a.weight.get_raw_value()).then(
                float_compare::cmp_f64(*a.chance.get_raw_value(), *b.chance.get_raw_value()),
            )
        });
    }

    Ok(data)
}

#[macro_export]
macro_rules! impl_common_group_analyzer_methods {
    () => {
        fn calculate_chance_for_group_step_index(
            &self,
            group_routes: &Vec<Vec<crate::calc::statistics::helpers::RouteChance>>,
            subpath_amount: crate::calc::statistics::helpers::SubpathAmount,
            index: usize,
        ) -> crate::calc::statistics::helpers::RouteChance {
            use crate::calc::statistics::helpers::RouteChance;

            // Sum all values found at this index across group routes
            let total: f64 = group_routes
                .iter()
                .filter_map(|gr| gr.get(index))
                .map(|rc| rc.get_raw_value())
                .sum();

            // Convert subpath_amount to f64
            let denom = (*subpath_amount.get_raw_value()) as f64;

            let value = if denom > 0.0 { total / denom } else { 0.0 };

            RouteChance::new(value.clamp(0.0, 1.0))
        }

        fn calculate_cost_per_craft(
            &self,
            currency: &Vec<crate::api::currency::CraftCurrencyList>,
            item_info: &crate::api::provider::item_info::ItemInfoProvider,
            market_provider: &crate::api::provider::market_prices::MarketPriceProvider,
        ) -> crate::api::provider::market_prices::PriceInDivines {
            let pc = crate::api::provider::market_prices::PriceInDivines::new(
                currency.iter().fold(0.0_f64, |a, b| {
                    a + b.list.iter().fold(0.0_f64, |a, b| {
                        a + market_provider
                            .try_lookup_currency_in_divines_default_if_fail(b, item_info)
                            .get_divine_value()
                    })
                }),
            );

            pc
        }

        fn calculate_tries_needed_for_60_percent(
            &self,
            group_route: &crate::api::calculator::GroupRoute,
        ) -> u64 {
            let tries_for_60 = ((((1.0_f64 - 0.6_f64).ln()
                / (1.0_f64 - group_route.chance.get_raw_value()).ln())
            .ceil()) as u64)
                .max(1);

            tries_for_60
        }
    };
}

#[macro_export]
macro_rules! impl_common_unique_path_analyzer_methods {
    () => {
        fn calculate_tries_needed_for_60_percent(
            &self,
            route: &crate::api::calculator::ItemRoute,
        ) -> u64 {
            let tries_for_60_percent = ((((1.0_f64 - 0.6_f64).ln()
                / (1.0_f64 - route.chance.get_raw_value()).ln())
            .ceil()) as u64)
                .max(1);

            tries_for_60_percent
        }

        fn calculate_cost_per_craft(
            &self,
            currency: &Vec<crate::api::currency::CraftCurrencyList>,
            item_info: &crate::api::provider::item_info::ItemInfoProvider,
            market_provider: &crate::api::provider::market_prices::MarketPriceProvider,
        ) -> crate::api::provider::market_prices::PriceInDivines {
            crate::api::provider::market_prices::PriceInDivines::new(currency.iter().fold(
                0.0_f64,
                |a, b| {
                    a + b.list.iter().fold(0.0_f64, |a, b| {
                        a + market_provider
                            .try_lookup_currency_in_divines_default_if_fail(b, item_info)
                            .get_divine_value()
                    })
                },
            ))
        }
    };
}
