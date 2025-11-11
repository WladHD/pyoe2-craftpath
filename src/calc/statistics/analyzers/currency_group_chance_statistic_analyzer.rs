use anyhow::Result;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, GroupRoute, StatisticAnalyzerCurrencyGroups},
        currency::CraftCurrencyList,
        provider::{
            item_info::ItemInfoProvider,
            market_prices::{MarketPriceProvider, PriceInDivines},
        },
        types::THashMap,
    },
    calc::statistics::{
        collectors::group_collector::CurrencyGroupChanceCollector,
        helpers::{RouteChance, RouteCustomWeight, StatisticAnalyzerCurrencyGroupCollectorTrait},
        statistic_analyzer_currency_grouped_collector::calculate_currency_groups,
    },
    utils::float_compare,
};

pub struct CurrencyGroupChanceStatisticAnalyzer;

impl StatisticAnalyzerCurrencyGroups for CurrencyGroupChanceStatisticAnalyzer {
    fn get_name(&self) -> &'static str {
        "Currency Groups by Highest Chance"
    }

    fn get_description(&self) -> &'static str {
        "Appends all possible paths to own currency sequences, allowing for a more general overview. Best combined with best N routes."
    }

    fn get_unit_type(&self) -> &'static str {
        "%"
    }

    fn lower_is_better(&self) -> bool {
        false
    }

    #[instrument(skip_all)]
    fn get_statistic(
        &self,
        calculator: &Calculator,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<GroupRoute>> {
        let res: THashMap<
            Vec<&CraftCurrencyList>,
            Vec<Vec<(RouteCustomWeight, RouteChance, u64)>>,
        > = calculate_currency_groups::<CurrencyGroupChanceCollector>(
            calculator,
            item_provider,
            market_provider,
            max_ram_in_bytes,
        )?;

        let mut data: Vec<GroupRoute> = res
            .into_par_iter()
            .map(|(k, v)| {
                let key_owned: Vec<CraftCurrencyList> = k.into_iter().cloned().collect();
                let weight = CurrencyGroupChanceCollector::calculate_group_weight(&key_owned, &v);
                let chance = CurrencyGroupChanceCollector::calculate_group_chance(&v);

                GroupRoute {
                    group: key_owned,
                    weight,
                    unique_route_weights: v,
                    chance,
                }
            })
            .collect();

        if self.lower_is_better() {
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

    fn format_display_more_info(
        &self,
        _: &GroupRoute,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Option<String> {
        None
    }

    fn calculate_chance_for_group_step_index(
        &self,
        group_routes: &Vec<Vec<(RouteCustomWeight, RouteChance, u64)>>,
        index: usize,
    ) -> RouteChance {
        let mut hm: THashMap<u64, Vec<RouteChance>> = THashMap::default();

        for gr in group_routes.iter() {
            let curr = gr.get(index).unwrap();
            hm.entry(curr.2.clone()).or_default().push(curr.1.clone());
        }

        let sum_max: f64 = hm.values().fold(0_f64, |acc, v| {
            acc + (v.iter().map(|e| *e.get_raw_value()).sum::<f64>() / (v.len() as f64))
        });

        RouteChance::new(sum_max.clamp(0_f64, 1_f64))
    }

    fn calculate_cost_per_craft(
        &self,
        currency: &Vec<CraftCurrencyList>,
        item_info: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
    ) -> PriceInDivines {
        let pc = PriceInDivines::new(currency.iter().fold(0_f64, |a, b| {
            a + b.list.iter().fold(0_f64, |a, b| {
                a + market_provider
                    .try_lookup_currency_in_divines_default_if_fail(b, &item_info)
                    .get_divine_value()
            })
        }));

        pc
    }

    fn calculate_tries_needed_for_60_percent(&self, group_route: &GroupRoute) -> u64 {
        let tries_for_60 = ((((1.0_f64 - 0.6_f64).ln()
            / (1.0_f64 - group_route.chance.get_raw_value()).ln())
        .ceil()) as u64)
            .max(1);

        tries_for_60
    }
}
