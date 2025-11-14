use std::mem::size_of;

use tracing::instrument;

use crate::{
    api::{
        calculator::{ItemMatrix, ItemMatrixNode, ItemRoute, ItemRouteNode},
        currency::CraftCurrencyList,
        item::ItemSnapshot,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
    },
    explicit_type,
    utils::{fraction_utils::Fraction, hash_utils::hash_value},
};

#[derive(Clone, Debug)]
pub struct ItemRouteNodeRef<'a> {
    pub item: &'a ItemSnapshot,
    pub chance: &'a Fraction,
    pub currency_list: &'a CraftCurrencyList,
}

#[derive(Clone, Debug)]
pub struct ItemRouteRef<'a> {
    pub route: Vec<ItemRouteNodeRef<'a>>,
    pub weight: RouteCustomWeight,
    pub chance: RouteChance,
}

/// Calculates RAM usage in bytes for an ItemRouteNodeRef<'a>.
pub fn ram_usage_item_route_node_ref<'a>(_node: &ItemRouteNodeRef<'a>) -> u64 {
    (size_of::<&ItemSnapshot>() + size_of::<&Fraction>() + size_of::<&CraftCurrencyList>()) as u64
}

/// Calculates RAM usage for an ItemRouteRef<'a> (includes Vec capacity).
pub fn ram_usage_item_route_ref<'a>(route_ref: &ItemRouteRef<'a>) -> u64 {
    let vec_capacity = route_ref.route.capacity();
    let node_size = size_of::<ItemRouteNodeRef<'a>>();
    let vec_overhead = size_of::<Vec<ItemRouteNodeRef<'a>>>();

    // route Vec + its allocated elements + f64 weight
    (vec_overhead + node_size * vec_capacity + size_of::<f64>()) as u64
}

/// Calculates RAM usage for a stack entry: (Vec<ItemRouteNodeRef>, &ItemMatrixNode)
pub fn ram_usage_stack_entry<'a>(path: &Vec<ItemRouteNodeRef<'a>>, _node: &ItemMatrixNode) -> u64 {
    let vec_capacity = path.capacity();
    let node_size = size_of::<ItemRouteNodeRef<'a>>();
    let vec_overhead = size_of::<Vec<ItemRouteNodeRef<'a>>>();
    let node_ref_size = size_of::<&ItemMatrixNode>();

    (vec_overhead + node_size * vec_capacity + node_ref_size) as u64
}

pub trait StatisticAnalyzerCollectorTrait {
    fn get_weight(
        path: &Vec<ItemRouteNodeRef<'_>>,
        matrix: &ItemMatrix,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
    ) -> (RouteCustomWeight, RouteChance);
}

explicit_type!(RouteCustomWeight, f64);
explicit_type!(SubpathAmount, u32);
explicit_type!(RouteChance, f64);

pub trait StatisticAnalyzerCurrencyGroupCollectorTrait {
    fn get_partial_weights(
        path: &Vec<ItemRouteNodeRef<'_>>,
        matrix: &ItemMatrix,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
    ) -> Vec<RouteChance>;

    fn calculate_group_weight(
        currency: &Vec<&CraftCurrencyList>,
        paths: &Vec<Vec<RouteChance>>,
    ) -> RouteCustomWeight;

    fn calculate_group_chance(paths: &Vec<Vec<RouteChance>>) -> RouteChance;
}

#[instrument(skip_all)]
pub fn finalize_routes(mut routes: Vec<ItemRouteRef<'_>>) -> Vec<ItemRoute> {
    tracing::info!("Collecting {} routes ...", routes.len());
    let mut finalized = Vec::new();

    for route_ref in routes.drain(..) {
        let mut owned_nodes = Vec::with_capacity(route_ref.route.len());
        for node_ref in route_ref.route {
            owned_nodes.push(ItemRouteNode {
                item_matrix_id: hash_value(node_ref.item),
                chance: node_ref.chance.clone(),
                currency_list: node_ref.currency_list.clone(),
            });
        }
        finalized.push(ItemRoute {
            route: owned_nodes,
            weight: route_ref.weight,
            chance: route_ref.chance,
        });
    }

    tracing::info!("Routes collected successfully.");
    finalized
}
