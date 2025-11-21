use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        calculator_utils::calculate_target_proximity::calculate_target_proximity,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{BaseGroupId, THashMap, THashSet},
    },
    utils::fraction_utils::Fraction,
};

static VAAL_ORB: &CraftCurrencyEnum = &CraftCurrencyEnum::VaalOrb();

static CORRUPTION_OMEN: &[Option<CraftCurrencyEnum>] =
    &[Some(CraftCurrencyEnum::OmenOfCorruption()), None];

pub struct VaalOrbSocketPropagator;

enum Category {
    SocketableEquipment,
    Ignore,
}

fn classify(id_bgroup: &BaseGroupId) -> Category {
    match id_bgroup.get_raw_value() {
        // Equipment
        2  // Body Armours
        | 3  // Boots
        | 5  // Gloves
        | 4  // Helmets
        | 8  // Offhands
        | 6  // One-Handed Weapons
        | 7  // Two-Handed Weapons
            => Category::SocketableEquipment,
        // Everything else
        _ => Category::Ignore,
    }
}

impl MatrixPropagator for VaalOrbSocketPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let base_group_id = provider.lookup_base_group(&item_instance.snapshot.base_id)?;

        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        if !matches!(classify(&base_group_id), Category::SocketableEquipment) {
            return Ok(propagation_result);
        }

        // only apply orb as very last step
        // (prefilter already takes care not to include socket amounts below max)
        if calculate_target_proximity(&item_instance.snapshot, &target, &provider)? != 1 {
            return Ok(propagation_result);
        }

        for corruption_omen in CORRUPTION_OMEN {
            let mut next_items: Vec<PropagationTarget> = Vec::new();

            let next_item_snapshot = ItemSnapshot {
                rarity: item_instance.snapshot.rarity.clone(),
                base_id: item_instance.snapshot.base_id.clone(),
                item_level: item_instance.snapshot.item_level.clone(),
                affixes: item_instance.snapshot.affixes.clone(),
                allowed_sockets: item_instance.snapshot.allowed_sockets.clone() + 1,
                corrupted: true,
                sockets: item_instance.snapshot.sockets.clone(),
            };
            let mut unique_currency_list = CraftCurrencyList {
                list: THashSet::default(),
            };

            // chance for non gems 25% (1 / 4) .. (1 / 3 with omen)
            // see https://www.poe2wiki.net/wiki/Corrupted
            match corruption_omen {
                Some(vaal_omen) => {
                    next_items.push(PropagationTarget::new(
                        Fraction::new(1, 3),
                        next_item_snapshot,
                    ));
                    unique_currency_list.list.insert(vaal_omen.clone());
                }
                None => {
                    next_items.push(PropagationTarget::new(
                        Fraction::new(1, 4),
                        next_item_snapshot,
                    ));
                }
            }

            unique_currency_list.list.insert(VAAL_ORB.clone());

            propagation_result.insert(unique_currency_list, next_items);
        }

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, provider: &ItemInfoProvider) -> bool {
        let Ok(base_group_id) = provider.lookup_base_group(&item.snapshot.base_id) else {
            return false;
        };

        let Ok(base_group_def) = provider.lookup_base_group_definition(&base_group_id) else {
            return false;
        };

        // only apply Vaal Orb as last step ... so check if max. natural sockets reached
        return base_group_def.max_sockets >= item.snapshot.allowed_sockets;
    }
}
