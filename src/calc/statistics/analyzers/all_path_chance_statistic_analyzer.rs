use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemRoute, StatisticAnalyzer},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::{
        collectors::chance_collector::UniquePathChanceCollector,
        helpers::ItemRouteRef,
        statistic_analyzer_all_path_collector::{calculate_all_paths, finalize_routes},
    },
};

pub struct AllUniquePathsChanceStatisticAnalyzer;

impl StatisticAnalyzer for AllUniquePathsChanceStatisticAnalyzer {
    fn get_name(&self) -> &'static str {
        "ALL Unique Paths by Highest Chance"
    }

    fn get_description(&self) -> &'static str {
        "Optimized to retrieves ALL unique paths from all possible combinations, sorted by chance. Uses a lot of memory for deep paths."
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
        _: u32,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<ItemRoute>> {
        let res: Vec<ItemRouteRef<'_>> = calculate_all_paths::<UniquePathChanceCollector>(
            calculator,
            item_provider,
            market_provider,
            max_ram_in_bytes,
            self.lower_is_better(),
        )?;

        Ok(finalize_routes(res))
    }
}
