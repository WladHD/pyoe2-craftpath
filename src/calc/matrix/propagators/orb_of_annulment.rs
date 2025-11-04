use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{AffixLocationEnum, AffixSpecifier, THashMap, THashSet},
    },
    utils::fraction_utils::Fraction,
};

static ANNUL_OMEN: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::DextralAnnulment()),
    Some(CraftCurrencyEnum::SinistralAnnulment()),
    None,
];

pub struct OrbOfAnnulmentPropagator;

impl MatrixPropagator for OrbOfAnnulmentPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        _: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        for omen in ANNUL_OMEN {
            let mut next_items: Vec<PropagationTarget> = Vec::new();
            let mut delete_item_affix_pool: THashSet<AffixSpecifier> =
                item_instance.snapshot.affixes.clone();

            delete_item_affix_pool.retain(|test| !test.fractured);

            match omen {
                Some(e) => match e {
                    CraftCurrencyEnum::DextralAnnulment() => {
                        delete_item_affix_pool.retain(|test| {
                            if let Ok(def) = provider.lookup_affix_definition(&test.affix) {
                                return def.affix_location == AffixLocationEnum::Suffix;
                            }

                            false
                        });
                    }

                    CraftCurrencyEnum::SinistralAnnulment() => {
                        delete_item_affix_pool.retain(|test| {
                            if let Ok(def) = provider.lookup_affix_definition(&test.affix) {
                                return def.affix_location == AffixLocationEnum::Prefix;
                            }

                            false
                        });
                    }
                    _ => {}
                },

                _ => {}
            };

            for target_affix_deletus in item_instance
                .helper
                .unwanted_affixes
                .iter()
                .filter(|test| delete_item_affix_pool.contains(&test))
            {
                let hit_chance_fraction = Fraction::new(1, delete_item_affix_pool.len() as u32);

                let mut cloned_affixed = item_instance.snapshot.affixes.clone();
                cloned_affixed.remove(target_affix_deletus);

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
                .insert(CraftCurrencyEnum::OrbOfAnnulment());

            match omen {
                Some(e) => {
                    unique_currency_list.list.insert(e.clone());
                }
                None => {}
            };

            propagation_result.insert(unique_currency_list, next_items);
        }

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, _provider: &ItemInfoProvider) -> bool {
        !item.helper.unwanted_affixes.is_empty()
    }
}
