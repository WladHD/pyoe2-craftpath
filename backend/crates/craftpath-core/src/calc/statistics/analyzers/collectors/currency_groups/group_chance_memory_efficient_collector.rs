use crate::{
    api::{
        calculator::ItemMatrix,
        currency::CraftCurrencyList,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    calc::statistics::helpers::{
        ItemRouteNodeRef, RouteChance, RouteCustomWeight,
        StatisticAnalyzerCurrencyGroupCollectorTrait,
    },
};

pub struct CurrencyGroupChanceMemoryEfficientCollector;

impl StatisticAnalyzerCurrencyGroupCollectorTrait for CurrencyGroupChanceMemoryEfficientCollector {
    fn get_partial_weights(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Vec<RouteChance> {
        path.iter().fold(Vec::new(), |mut a, b| {
            a.push(RouteChance::new(b.chance.to_f64()));
            a
        })
    }

    // in this case we sort by chance, so weight = chance
    fn calculate_group_weight(
        _: &Vec<&CraftCurrencyList>,
        paths: &Vec<Vec<RouteChance>>,
    ) -> RouteCustomWeight {
        RouteCustomWeight::from(
            paths
                .iter()
                .map(|e| e.iter().map(|e| *e.get_raw_value()).product::<f64>())
                .sum::<f64>(),
        )
    }

    fn calculate_group_chance(paths: &Vec<Vec<RouteChance>>) -> RouteChance {
        RouteChance::from(
            paths
                .iter()
                .map(|e| e.iter().map(|e| *e.get_raw_value()).product::<f64>())
                .sum::<f64>(),
        )
    }
}
