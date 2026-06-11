use crate::{
    api::{
        calculator::ItemMatrix,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, RouteChance, RouteCustomWeight, StatisticAnalyzerCollectorTrait,
    },
};

pub struct UniquePathChanceCollector;

impl StatisticAnalyzerCollectorTrait for UniquePathChanceCollector {
    fn get_weight(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> (RouteCustomWeight, RouteChance) {
        let res = path.iter().map(|n| n.chance.to_f64()).product::<f64>();
        (RouteCustomWeight::from(res), RouteChance::from(res))
    }
}
