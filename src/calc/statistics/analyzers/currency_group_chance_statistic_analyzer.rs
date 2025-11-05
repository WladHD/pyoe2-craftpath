use anyhow::Result;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemMatrix, StatisticAnalyzerCurrencyGroups},
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
    ) -> Result<Vec<(Vec<CraftCurrencyList>, f64, Vec<Vec<f64>>)>> {
        let res: THashMap<Vec<&CraftCurrencyList>, Vec<Vec<f64>>> =
            calculate_currency_groups::<UniquePathChanceCollector>(
                calculator,
                item_provider,
                market_provider,
                max_ram_in_bytes,
            )?;

        let mut data: Vec<(Vec<CraftCurrencyList>, f64, Vec<Vec<f64>>)> = res
            .into_par_iter()
            .map(|(k, v)| {
                let key_owned: Vec<CraftCurrencyList> = k.into_iter().cloned().collect();
                let weight = UniquePathChanceCollector::calculate_group_weight(&key_owned, &v);
                (key_owned, weight, v)
            })
            .collect();

        if self.lower_is_better() {
            data.par_sort_unstable_by(|a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            data.par_sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        Ok(data)
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
