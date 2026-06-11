use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemRoute, StatisticAnalyzerPaths},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::{
        analyzers::collectors::{
            unique_paths::efficient_cost_collector::UniquePathEfficientCostCollector,
            utils::statistic_analyzer_unique_collector::calculate_crafting_paths,
        },
        helpers::{ItemRouteRef, finalize_routes},
    },
    impl_common_unique_path_analyzer_methods,
};

pub struct UniquePathEfficientCostStatisticAnalyzer;

impl StatisticAnalyzerPaths for UniquePathEfficientCostStatisticAnalyzer {
    fn get_name(&self) -> &'static str {
        "Unique Path by Efficient Cost"
    }

    fn get_description(&self) -> &'static str {
        "Retrieves N number of unique paths memory efficiently from all possible combinations, sorted by cost multiplied by amount of tries needed to reach at least 60% chance to gain the item."
    }

    fn get_unit_type(&self) -> &'static str {
        "EX"
    }

    fn lower_is_better(&self) -> bool {
        true
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
        let res: Vec<ItemRouteRef<'_>> =
            calculate_crafting_paths::<UniquePathEfficientCostCollector>(
                calculator,
                item_provider,
                market_provider,
                max_routes,
                max_ram_in_bytes,
                self.lower_is_better(),
            )?;

        Ok(finalize_routes(res))
    }

    fn format_display_more_info(
        &self,
        _: &crate::api::calculator::ItemRoute,
        _: &crate::api::provider::item_info::ItemInfoProvider,
        _: &crate::api::provider::market_prices::MarketPriceProvider,
    ) -> Option<String> {
        None
    }

    impl_common_unique_path_analyzer_methods!();
}
