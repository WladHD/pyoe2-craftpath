use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemRoute, StatisticAnalyzerPaths},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::{
        analyzers::collectors::{
            unique_paths::chance_collector::UniquePathChanceCollector,
            utils::statistic_analyzer_unique_collector::calculate_crafting_paths,
        },
        helpers::{ItemRouteRef, finalize_routes},
    },
    impl_common_unique_path_analyzer_methods,
};

pub struct UniquePathChanceStatisticAnalyzer;

impl StatisticAnalyzerPaths for UniquePathChanceStatisticAnalyzer {
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
