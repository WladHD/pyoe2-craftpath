use anyhow::{Result, anyhow};

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{AffixLocationEnum, AffixSpecifier, ItemRarityEnum, THashMap, THashSet},
    },
    calc::matrix::propagators::exalted_orb::ExaltedOrbPropagator,
    utils::fraction_utils::Fraction,
};

static CHAOS_ORBS: &[CraftCurrencyEnum] = &[
    CraftCurrencyEnum::ChaosOrbNormal(),
    CraftCurrencyEnum::ChaosOrbGreater(),
    CraftCurrencyEnum::ChaosOrbPerfect(),
];

static CHAOS_OMEN_LOCATION: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::DextralErasure()),
    Some(CraftCurrencyEnum::SinistralErasure()),
    None,
];

static CHAOS_OMEN_WHITTLING: &[Option<CraftCurrencyEnum>] =
    &[Some(CraftCurrencyEnum::Whittling()), None];

pub struct ChaosOrbPropagator;

impl MatrixPropagator for ChaosOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let mut lowest_pool = item_instance.snapshot.affixes.clone();
        // keep in mind higher = worse -> T1 vs T5
        if let Some(min_tier) = lowest_pool
            .iter()
            .filter_map(|test| match !test.fractured {
                true => None,
                false => Some(test.tier.tier.clone()),
            })
            .max()
        {
            lowest_pool.retain(|test| test.tier.tier == min_tier && !test.fractured);
        }

        for chaos_orb in CHAOS_ORBS {
            for whittling_omen in CHAOS_OMEN_WHITTLING {
                for location_omen in CHAOS_OMEN_LOCATION {
                    let mut next_items: Vec<PropagationTarget> = Vec::new();
                    let mut delete_item_affix_pool: THashSet<AffixSpecifier> =
                        item_instance.helper.unwanted_affixes.clone();

                    delete_item_affix_pool.retain(|test| !test.fractured);

                    match location_omen {
                        Some(e) => match e {
                            CraftCurrencyEnum::DextralErasure() => {
                                let is_suffix = |test: &AffixSpecifier| {
                                    provider
                                        .lookup_affix_definition(&test.affix)
                                        .map(|def| def.affix_location == AffixLocationEnum::Suffix)
                                        .unwrap_or(false)
                                };

                                lowest_pool.retain(is_suffix);
                                delete_item_affix_pool.retain(is_suffix);
                            }

                            CraftCurrencyEnum::SinistralErasure() => {
                                let is_prefix = |test: &AffixSpecifier| {
                                    provider
                                        .lookup_affix_definition(&test.affix)
                                        .map(|def| def.affix_location == AffixLocationEnum::Prefix)
                                        .unwrap_or(false)
                                };

                                lowest_pool.retain(is_prefix);
                                delete_item_affix_pool.retain(is_prefix);
                            }
                            _ => {}
                        },

                        _ => {}
                    };

                    match whittling_omen {
                        Some(_) => delete_item_affix_pool.retain(|test| lowest_pool.contains(test)),
                        None => {}
                    };

                    for target_affix_deletus in delete_item_affix_pool.iter() {
                        let hit_chance_fraction =
                            Fraction::new(1, delete_item_affix_pool.len() as u32);

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

                        let next_item_instance =
                            Item::build_with(next_item_snapshot, &target_item, &provider)?;

                        let mut propagations = ExaltedOrbPropagator::propagate_step_explicit(
                            match chaos_orb {
                                CraftCurrencyEnum::ChaosOrbNormal() => {
                                    &CraftCurrencyEnum::ExaltedOrbNormal()
                                }
                                CraftCurrencyEnum::ChaosOrbGreater() => {
                                    &CraftCurrencyEnum::ExaltedOrbGreater()
                                }
                                CraftCurrencyEnum::ChaosOrbPerfect() => {
                                    &CraftCurrencyEnum::ExaltedOrbPerfect()
                                }
                                _ => return Err(anyhow!("Wrong currency")),
                            },
                            None,
                            None,
                            &next_item_instance,
                            &target_item,
                            None,
                            &provider,
                        )?;

                        for prop in propagations.iter_mut() {
                            prop.chance = prop.chance * hit_chance_fraction;
                        }

                        next_items.extend(propagations);
                    }

                    let mut prop_id = CraftCurrencyList {
                        list: THashSet::default(),
                    };

                    prop_id.list.insert(chaos_orb.clone());

                    if let Some(whit) = whittling_omen {
                        prop_id.list.insert(whit.clone());
                    }

                    if let Some(loc) = location_omen {
                        prop_id.list.insert(loc.clone());
                    }

                    propagation_result.insert(prop_id, next_items);
                }
            }
        }

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, _provider: &ItemInfoProvider) -> bool {
        match item.snapshot.rarity {
            ItemRarityEnum::Rare => !item.helper.unwanted_affixes.is_empty(),
            _ => false,
        }
    }
}
