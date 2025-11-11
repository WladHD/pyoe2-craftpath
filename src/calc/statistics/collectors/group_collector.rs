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
    utils::hash_utils::hash_value,
};

pub struct CurrencyGroupChanceCollector;

impl StatisticAnalyzerCurrencyGroupCollectorTrait for CurrencyGroupChanceCollector {
    fn get_partial_weights(
        path: &Vec<ItemRouteNodeRef<'_>>,
        _: &ItemMatrix,
        _: &ItemInfoProvider,
        _: &MarketPriceProvider,
    ) -> Vec<(RouteCustomWeight, RouteChance, u64)> {
        path.iter().fold(Vec::new(), |mut a, b| {
            a.push((
                RouteCustomWeight::new(b.chance.to_f64()),
                RouteChance::new(b.chance.to_f64()),
                hash_value(b.item),
            ));
            a
        })
    }

    fn calculate_group_weight(
        _: &Vec<CraftCurrencyList>,
        paths: &Vec<Vec<(RouteCustomWeight, RouteChance, u64)>>,
    ) -> RouteCustomWeight {
        RouteCustomWeight::from(
            paths
                .iter()
                .map(|e| e.iter().map(|e| *e.0.get_raw_value()).product::<f64>())
                .sum::<f64>(),
        )
    }

    fn calculate_group_chance(
        paths: &Vec<Vec<(RouteCustomWeight, RouteChance, u64)>>,
    ) -> RouteChance {
        RouteChance::from(
            paths
                .iter()
                .map(|e| e.iter().map(|e| *e.1.get_raw_value()).product::<f64>())
                .sum::<f64>(),
        )
    }
}
