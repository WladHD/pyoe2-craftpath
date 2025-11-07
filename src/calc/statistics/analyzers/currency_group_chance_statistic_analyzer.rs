use anyhow::Result;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, GroupRoute, ItemMatrix, StatisticAnalyzerCurrencyGroups},
        currency::CraftCurrencyList,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashMap,
    },
    calc::statistics::{
        helpers::{ItemRouteNodeRef, StatisticAnalyzerCurrencyGroupCollectorTrait},
        statistic_analyzer_currency_grouped_collector::calculate_currency_groups,
    },
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
        let res: THashMap<Vec<&CraftCurrencyList>, Vec<Vec<f64>>> =
            calculate_currency_groups::<UniquePathChanceCollector>(
                calculator,
                item_provider,
                market_provider,
                max_ram_in_bytes,
            )?;

        let mut data: Vec<GroupRoute> = res
            .into_par_iter()
            .map(|(k, v)| {
                let key_owned: Vec<CraftCurrencyList> = k.into_iter().cloned().collect();
                let weight = UniquePathChanceCollector::calculate_group_weight(&key_owned, &v);

                GroupRoute {
                    group: key_owned,
                    weight: weight,
                    unique_route_weights: v,
                }
            })
            .collect();

        if self.lower_is_better() {
            data.par_sort_unstable_by(|a, b| {
                a.weight
                    .partial_cmp(&b.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            data.par_sort_unstable_by(|a, b| {
                b.weight
                    .partial_cmp(&a.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        Ok(data)
    }

    fn calculate_weight_for_60_percent(
        &self,
        route: &GroupRoute,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> f64 {
        let tries_for_60_percent =
            (((1.0_f64 - 0.6_f64).ln() / (1.0_f64 - route.weight).ln()).ceil()).max(1_f64);

        tries_for_60_percent
    }

    fn template_group_weight_name(&self) -> &'static str {
        "Chance"
    }

    fn template_60_percent_group_name(&self) -> &'static str {
        "Tries needed for"
    }

    fn format_group_weight(&self, weight: f64) -> String {
        format!("{:.5} %", weight * 100_f64)
    }

    fn format_60_percent_group_weight(&self, weight: f64) -> String {
        format!("{} tries", weight as u64)
    }

    fn format_display_more_info(
        &self,
        _: &GroupRoute,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Option<String> {
        None
    }

    fn calculate_weight_for_group_step_index(
        &self,
        group_routes: &Vec<Vec<f64>>,
        index: usize,
    ) -> f64 {
        let route_weights: Vec<f64> = group_routes
            .iter()
            .map(|route| route.iter().product::<f64>())
            .collect();

        let total_weight: f64 = route_weights.iter().sum();

        // weighted sum of the probability at this step
        let step_weight: f64 = group_routes
            .iter()
            .zip(route_weights.iter())
            .map(|(route, w)| route[index] * w)
            .sum();

        step_weight / total_weight
    }

    fn template_weight_for_group_step_index(&self, weight: f64) -> String {
        format!("{:.5} %", weight * 100_f64)
    }
}

struct UniquePathChanceCollector;

impl StatisticAnalyzerCurrencyGroupCollectorTrait for UniquePathChanceCollector {
    fn get_partial_weights(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Vec<f64> {
        path.iter().fold(Vec::new(), |mut a, b| {
            a.push(b.chance.to_f64());
            a
        })
    }

    fn calculate_group_weight(_: &Vec<CraftCurrencyList>, paths: &Vec<Vec<f64>>) -> f64 {
        paths.iter().map(|e| e.iter().product::<f64>()).sum::<f64>()
    }
}
