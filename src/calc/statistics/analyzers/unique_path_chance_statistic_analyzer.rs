use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemMatrix, ItemRoute, StatisticAnalyzer},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::{
        helpers::{ItemRouteNodeRef, ItemRouteRef},
        statistic_analyzer_all_path_collector::finalize_routes,
        statistic_analyzer_unique_collector::{
            StatisticAnalyzerCollectorTrait, calculate_crafting_paths,
        },
    },
};

pub struct UniquePathChanceStatisticAnalyzer;

impl StatisticAnalyzer for UniquePathChanceStatisticAnalyzer {
    fn get_name(&self) -> &'static str {
        "Unique Path by Highest Chance"
    }

    fn get_description(&self) -> &'static str {
        "Retrieves N number of unique paths memory efficiently from all possible combinations, sorted by chance."
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
        max_routes: u32,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<ItemRoute>> {
        let res: Vec<ItemRouteRef<'_>> = calculate_crafting_paths::<UniquePathChanceCollector>(
            calculator,
            item_provider,
            market_provider,
            max_routes,
            max_ram_in_bytes,
            self.lower_is_better(),
        )?;

        Ok(finalize_routes(res))
    }
}

struct UniquePathChanceCollector;

impl StatisticAnalyzerCollectorTrait for UniquePathChanceCollector {
    fn get_weight(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> f64 {
        path.iter().map(|n| n.chance.to_f64()).product()
    }
}
