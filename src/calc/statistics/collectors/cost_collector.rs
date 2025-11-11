use crate::{
    api::{
        calculator::ItemMatrix,
        provider::{
            item_info::ItemInfoProvider,
            market_prices::{MarketPriceProvider, PriceKind},
        },
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, RouteChance, RouteCustomWeight, StatisticAnalyzerCollectorTrait,
    },
};

pub struct UniquePathCostCollector;

impl StatisticAnalyzerCollectorTrait for UniquePathCostCollector {
    fn get_weight(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        item_info: &ItemInfoProvider,
        market_info: &MarketPriceProvider,
    ) -> (RouteCustomWeight, RouteChance) {
        let chance = path.iter().map(|n| n.chance.to_f64()).product::<f64>();

        let cost_total = path
            .iter()
            .map(|n| {
                n.currency_list.list.iter().fold(0_f64, |a, b| {
                    a + market_info.currency_convert(
                        &market_info.try_lookup_currency_in_divines_default_if_fail(&b, &item_info),
                        &PriceKind::Exalted,
                    )
                })
            })
            .sum::<f64>();
        (
            RouteCustomWeight::from(cost_total),
            RouteChance::from(chance),
        )
    }
}
