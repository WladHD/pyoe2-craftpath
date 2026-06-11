use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        calculator_utils::calculate_target_proximity::calculate_target_proximity_affixes,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{THashMap, THashSet},
    },
    utils::fraction_utils::Fraction,
};

static ART_ORB: &CraftCurrencyEnum = &CraftCurrencyEnum::ArtificersOrb();

pub struct ArtificersOrbPropagator;

impl MatrixPropagator for ArtificersOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        if target.allowed_sockets <= item_instance.snapshot.allowed_sockets {
            return Ok(propagation_result);
        }

        // only apply orb if affixes are finished
        if calculate_target_proximity_affixes(&item_instance.snapshot, &target, &provider)? != 0_u8
        {
            return Ok(propagation_result);
        }

        let mut next_items: Vec<PropagationTarget> = Vec::new();

        let next_item_snapshot = ItemSnapshot {
            rarity: item_instance.snapshot.rarity.clone(),
            base_id: item_instance.snapshot.base_id.clone(),
            item_level: item_instance.snapshot.item_level.clone(),
            affixes: item_instance.snapshot.affixes.clone(),
            allowed_sockets: item_instance.snapshot.allowed_sockets.clone() + 1,
            corrupted: item_instance.snapshot.corrupted.clone(),
            sockets: item_instance.snapshot.sockets.clone(),
        };

        let next_item = PropagationTarget::new(Fraction::one(), next_item_snapshot);

        next_items.push(next_item);

        let mut unique_currency_list = CraftCurrencyList {
            list: THashSet::default(),
        };
        unique_currency_list.list.insert(ART_ORB.clone());

        propagation_result.insert(unique_currency_list, next_items);

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, provider: &ItemInfoProvider) -> bool {
        let Ok(base_group_id) = provider.lookup_base_group(&item.snapshot.base_id) else {
            return false;
        };

        let Ok(base_group_def) = provider.lookup_base_group_definition(&base_group_id) else {
            return false;
        };

        if base_group_def.max_sockets <= item.snapshot.allowed_sockets {
            return false;
        }

        return true;
    }
}
