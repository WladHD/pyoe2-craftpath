use crate::{
    api::{
        calculator::ItemMatrix,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::{
        helpers::ItemRouteNodeRef,
        statistic_analyzer_all_path_collector::StatisticAnalyzerCollectorTrait,
    },
};

pub struct UniquePathChanceCollector;

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
