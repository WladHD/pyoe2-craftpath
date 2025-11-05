#[derive(Clone, Debug)]
pub struct ItemRouteNodeRef<'a> {
    pub item: &'a ItemSnapshot,
    pub chance: &'a Fraction,
    pub currency_list: &'a CraftCurrencyList,
}

#[derive(Clone, Debug)]
pub struct ItemRouteRef<'a> {
    pub route: Vec<ItemRouteNodeRef<'a>>,
    pub weight: f64,
}

use std::mem::size_of;

use crate::{
    api::{calculator::ItemMatrixNode, currency::CraftCurrencyList, item::ItemSnapshot},
    utils::fraction_utils::Fraction,
};

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
