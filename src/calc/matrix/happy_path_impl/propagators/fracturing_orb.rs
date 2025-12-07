use anyhow::Result;
use tracing::info;

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{AffixSpecifier, ItemRarityEnum, THashMap, THashSet},
    },
    utils::fraction_utils::Fraction,
};

pub struct FracturingOrbPropagator;

impl MatrixPropagator for FracturingOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        _: &ItemSnapshot,
        _: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let wanted_item_affixes: THashSet<&AffixSpecifier> = item_instance
            .snapshot
            .affixes
            .difference(&item_instance.helper.unwanted_affixes)
            .collect();

        let mut next_items: Vec<PropagationTarget> = Vec::new();

        for to_be_fractured in wanted_item_affixes {
            info!("{:?}", to_be_fractured);
            let hit_chance_fraction = Fraction::new(1, item_instance.snapshot.affixes.len() as u32);

            let mut cloned_affixed = item_instance.snapshot.affixes.clone();
            let mut affix = cloned_affixed.take(&to_be_fractured).unwrap();
            affix.fractured = true;
            cloned_affixed.insert(affix);

            let next_item_snapshot = ItemSnapshot {
                rarity: item_instance.snapshot.rarity.clone(),
                base_id: item_instance.snapshot.base_id.clone(),
                item_level: item_instance.snapshot.item_level.clone(),
                affixes: cloned_affixed,
                allowed_sockets: item_instance.snapshot.allowed_sockets.clone(),
                corrupted: item_instance.snapshot.corrupted.clone(),
                sockets: item_instance.snapshot.sockets.clone(),
            };

            let next_item = PropagationTarget::new(hit_chance_fraction, next_item_snapshot);

            next_items.push(next_item);
        }

        let mut unique_currency_list = CraftCurrencyList {
            list: THashSet::default(),
        };

        unique_currency_list
            .list
            .insert(CraftCurrencyEnum::FracturingOrb());

        propagation_result.insert(unique_currency_list, next_items);

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, _provider: &ItemInfoProvider) -> bool {
        match item.snapshot.rarity {
            ItemRarityEnum::Rare if item.snapshot.affixes.len() == 4 => {
                item.snapshot.affixes.iter().all(|test| !test.fractured)
            }
            _ => false,
        }
    }
}
