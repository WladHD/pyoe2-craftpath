use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, GroupRoute, StatisticAnalyzerCurrencyGroups},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::analyzers::{
        collectors::currency_groups::group_chance_memory_efficient_collector::CurrencyGroupChanceMemoryEfficientCollector,
        common_analyzer_utils::get_grouped_statistic_memory_efficient,
    },
    impl_common_group_analyzer_methods,
};

pub struct CurrencyGroupChanceMemoryEfficientStatisticAnalyzer;

impl StatisticAnalyzerCurrencyGroups for CurrencyGroupChanceMemoryEfficientStatisticAnalyzer {
    fn get_name(&self) -> &'static str {
        "Currency Groups by Highest Chance (No Unique Paths)"
    }

    fn get_description(&self) -> &'static str {
        "Memory efficient implementation of currency sequence grouping. \
        Unique paths are not kept, but instead immediatly summed, thus losing information \
        but allowing memory efficient collection. Best combined with best N routes."
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
        get_grouped_statistic_memory_efficient::<CurrencyGroupChanceMemoryEfficientCollector>(
            self.lower_is_better(),
            calculator,
            item_provider,
            market_provider,
            max_ram_in_bytes,
        )
    }

    fn format_display_more_info(
        &self,
        _: &GroupRoute,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Option<String> {
        None
    }

    impl_common_group_analyzer_methods!();
}
