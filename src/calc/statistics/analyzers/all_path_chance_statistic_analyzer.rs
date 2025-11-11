use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        calculator::{Calculator, ItemRoute, StatisticAnalyzerPaths},
        currency::CraftCurrencyList,
        provider::{
            item_info::ItemInfoProvider,
            market_prices::{MarketPriceProvider, PriceInDivines},
        },
    },
    calc::statistics::{
        collectors::chance_collector::UniquePathChanceCollector,
        helpers::{ItemRouteRef, finalize_routes},
        statistic_analyzer_all_path_collector::calculate_all_paths,
    },
};

pub struct AllUniquePathsChanceStatisticAnalyzer;

impl StatisticAnalyzerPaths for AllUniquePathsChanceStatisticAnalyzer {
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

    fn calculate_tries_needed_for_60_percent(&self, route: &ItemRoute) -> u64 {
        let tries_for_60_percent = ((((1.0_f64 - 0.6_f64).ln()
            / (1.0_f64 - route.chance.get_raw_value()).ln())
        .ceil()) as u64)
            .max(1);

        tries_for_60_percent
    }

    fn format_display_more_info(
        &self,
        _: &ItemRoute,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Option<String> {
        None
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
}
