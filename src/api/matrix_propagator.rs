use anyhow::Result;

use crate::api::{
    calculator::PropagationTarget,
    currency::CraftCurrencyList,
    item::{Item, ItemSnapshot},
    provider::item_info::ItemInfoProvider,
    types::{AffixSpecifier, THashMap, THashSet},
};

pub trait MatrixPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>>;

    fn is_applicable(&self, item: &Item, provider: &ItemInfoProvider) -> bool;
}
