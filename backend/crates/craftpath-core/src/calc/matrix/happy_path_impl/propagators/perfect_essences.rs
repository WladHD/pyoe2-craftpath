use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        errors::CraftPathError,
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
            AffixTierLevelBoundsEnum, ItemRarityEnum, THashMap, THashSet,
        },
    },
    calc::matrix::happy_path_impl::propagators::exalted_orb::ExaltedOrbPropagator,
    utils::fraction_utils::Fraction,
};

static DEX_SIN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::SinistralCrystallisation()),
    Some(CraftCurrencyEnum::DextralCrystallisation()),
    None,
];

pub struct PerfectEssencePropagator;

impl PerfectEssencePropagator {
    pub fn propagate_step_explicit(
        currency: &CraftCurrencyEnum,
        affix_target: &AffixSpecifier,
        dex_sin: Option<&CraftCurrencyEnum>,
        item_instance: &Item,
        provider: &ItemInfoProvider,
    ) -> Result<Option<Vec<PropagationTarget>>> {
        let target_essence_def = match &currency {
            &CraftCurrencyEnum::Essence(e) => {
                let Ok(def) = provider.lookup_essence_definition(&e) else {
                    return Ok(None);
                };
                def
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown currency"));
            }
        };

        if !target_essence_def.name_essence.starts_with("Perfect") {
            return Ok(None);
        }

        let min_lvl = &target_essence_def
            .base_tier_table
            .get(&item_instance.snapshot.base_id)
            .unwrap()
            .get(&affix_target.affix)
            .unwrap()
            .min_item_level;

        if min_lvl > &item_instance.snapshot.item_level {
            return Err(CraftPathError::ItemUnreachableMinLevelConstraint(
                min_lvl.clone(),
                item_instance.snapshot.item_level.clone(),
                affix_target.affix.clone(),
            )
            .into());
        }

        let affix_target_def = provider
            .lookup_affix_definition(&affix_target.affix)
            .unwrap();

        if item_instance
            .snapshot
            .affixes
            .iter()
            .any(|item_affixes| &item_affixes.affix == &affix_target.affix)
        {
            return Ok(None);
        }

        let blocked_mod_groups_affix = &affix_target_def.exlusive_groups;

        if !blocked_mod_groups_affix.is_disjoint(&item_instance.helper.blocked_modgroups) {
            // should this be handled differently?
            return Ok(None);
        }

        let mut next_items: Vec<PropagationTarget> = Vec::new();

        let mut delete_item_affix_pool: THashSet<AffixSpecifier> =
            item_instance.snapshot.affixes.clone();

        delete_item_affix_pool.retain(|test| !test.fractured);

        // filter unwanted pool
        match dex_sin {
            Some(e) => match e {
                CraftCurrencyEnum::DextralCrystallisation() => {
                    if affix_target_def.affix_location != AffixLocationEnum::Suffix {
                        return Ok(None);
                    }

                    delete_item_affix_pool.retain(|test| {
                        if let Ok(def) = provider.lookup_affix_definition(&test.affix) {
                            return def.affix_location == AffixLocationEnum::Suffix;
                        }

                        false
                    });
                }

                CraftCurrencyEnum::SinistralCrystallisation() => {
                    if affix_target_def.affix_location != AffixLocationEnum::Prefix {
                        return Ok(None);
                    }

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

        let want_to_delete = item_instance
            .helper
            .unwanted_affixes
            .iter()
            .filter(|test| delete_item_affix_pool.contains(&test))
            .collect::<Vec<_>>();

        let (unwanted_prefix_count, unwanted_suffix_count) = {
            let mut prefix = 0u8;
            let mut suffix = 0u8;

            for test in want_to_delete.iter() {
                match provider
                    .lookup_affix_definition(&test.affix)
                    .unwrap()
                    .affix_location
                {
                    AffixLocationEnum::Prefix => prefix += 1,
                    AffixLocationEnum::Suffix => suffix += 1,
                    _ => {}
                }
            }

            (prefix, suffix)
        };

        let combined_unwanted_affixes_count = unwanted_suffix_count + unwanted_prefix_count;

        // CHECK IF AN UNWANTED MOD CAN BE DELETED
        // we can always add one mod, but if all mods are full and all mods are wanted, we can't (happy path)
        if let Some((location, count, unwanted_count)) = match affix_target_def.affix_location {
            AffixLocationEnum::Prefix => Some((
                AffixLocationEnum::Prefix,
                item_instance.helper.prefix_count,
                unwanted_prefix_count,
            )),
            AffixLocationEnum::Suffix => Some((
                AffixLocationEnum::Suffix,
                item_instance.helper.suffix_count,
                unwanted_suffix_count,
            )),
            _ => None,
        } {
            // insta abort, nothing we can solve
            if count == 3 && unwanted_count == 0 {
                return Err(CraftPathError::ItemUnreachable(
                    item_instance.clone(),
                    affix_target.clone(),
                )
                .into());
            }

            if combined_unwanted_affixes_count == 0 {
                // TODO: SPECIAL CASE - ADD LOOSE MODS =>
                // apply ExaltedOrb with different omens and no propagation restrictions once -> maybe only set min tier to max
                return Err(CraftPathError::EssenceIntermediaryStepRequired(
                    affix_target_def.affix_location.clone(),
                )
                .into());
            }

            // if all affixes are full and we have unwanted ones, it is now forced
            if count == 3 {
                delete_item_affix_pool.retain(|test| {
                    provider
                        .lookup_affix_definition(&test.affix)
                        .unwrap()
                        .affix_location
                        == location
                });
            }
        }

        for delete_affix in want_to_delete
            .iter()
            .filter(|test| delete_item_affix_pool.contains(test))
        {
            let mut affixes: THashSet<AffixSpecifier> = item_instance.snapshot.affixes.clone();
            affixes.remove(delete_affix);

            let Ok((_, _, essence_tier)) = provider.collect_essence_info_for_affix(
                match currency {
                    CraftCurrencyEnum::Essence(e) => e,
                    _ => panic!("Unknown currency"),
                },
                &item_instance.snapshot.base_id,
                &affix_target.affix,
            ) else {
                continue;
            };

            affixes.insert(AffixSpecifier {
                affix: affix_target.affix.clone(),
                fractured: false,
                tier: AffixTierConstraints {
                    bounds: AffixTierLevelBoundsEnum::Exact,
                    tier: essence_tier.0.clone(),
                },
            });

            // affixes.insert(affix_target.clone());

            let next_item_snapshot = ItemSnapshot {
                rarity: item_instance.snapshot.rarity.clone(),
                base_id: item_instance.snapshot.base_id.clone(),
                item_level: item_instance.snapshot.item_level.clone(),
                affixes: affixes,
                allowed_sockets: item_instance.snapshot.allowed_sockets.clone(),
                corrupted: item_instance.snapshot.corrupted.clone(),
                sockets: item_instance.snapshot.sockets.clone(),
            };

            let next_item = PropagationTarget::new(
                Fraction::new(1, delete_item_affix_pool.len() as u32),
                next_item_snapshot,
            );

            next_items.push(next_item);
        }

        Ok(match next_items.is_empty() {
            true => None,
            false => Some(next_items),
        })
    }
}

// TODO optimize omen use.. if no change is observed while using it, dont propagate that route
impl MatrixPropagator for PerfectEssencePropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let target_affixes = &target_item.affixes;

        let open_essence_mods = target_affixes
            .iter()
            .filter(|test| {
                match provider
                    .lookup_affix_definition(&test.affix)
                    .unwrap()
                    .affix_class
                {
                    AffixClassEnum::Essence | AffixClassEnum::Base => true,
                    _ => false,
                }
            })
            // check that essence is not contained in item already
            .filter(|test| {
                item_instance
                    .snapshot
                    .affixes
                    .iter()
                    .all(|i| i.affix != test.affix)
            })
            .filter_map(|e| {
                let Ok(ad) = provider.lookup_affix_definition(&e.affix) else {
                    return None;
                };

                let Ok(ae) = provider
                    .lookup_affix_essences(e.affix.clone(), item_instance.snapshot.base_id.clone())
                else {
                    return None;
                };

                Some((e, ad, ae))
            });

        let mut request_temp_step = THashSet::default();

        'outer: for (spec, _, ess_def) in open_essence_mods {
            for dex_sin in DEX_SIN_OMEN_GROUP {
                for ess_def in ess_def {
                    let curr = CraftCurrencyEnum::Essence(ess_def.clone());

                    match PerfectEssencePropagator::propagate_step_explicit(
                        &CraftCurrencyEnum::Essence(ess_def.clone()),
                        spec,
                        dex_sin.as_ref(),
                        item_instance,
                        &provider,
                    ) {
                        Ok(next_items) => {
                            let Some(next_items) = next_items else {
                                continue;
                            };

                            let mut unique_currency_list = CraftCurrencyList {
                                list: THashSet::default(),
                            };

                            if let Some(dex_sin) = dex_sin {
                                unique_currency_list.list.insert(dex_sin.clone());
                            }

                            unique_currency_list.list.insert(curr);
                            propagation_result.insert(unique_currency_list, next_items);
                        }

                        Err(e) => {
                            propagation_result.clear();

                            if let Some(CraftPathError::EssenceIntermediaryStepRequired(location)) =
                                e.downcast_ref::<CraftPathError>()
                            {
                                request_temp_step.insert((location.clone(), spec, ess_def));
                                break 'outer;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }

        if !request_temp_step.is_empty() && !item_instance.meta.mark_for_essence_only {
            let mut locs = THashSet::default();
            request_temp_step.iter().for_each(|e| {
                locs.insert(e.0.clone());
            });

            let mut actual_props_with_temp: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
                THashMap::default();

            for loc in locs {
                let results = ExaltedOrbPropagator::propagate_step_unwanted_location_bound(
                    item_instance,
                    target_item,
                    &loc,
                    &provider,
                )?;

                for (key, mut vals) in results {
                    actual_props_with_temp
                        .entry(key)
                        .and_modify(|existing| {
                            existing.extend(vals.drain(..));
                            // deduplicate just in case
                            let mut seen = THashSet::default();
                            existing.retain(|x| seen.insert(x.clone()));
                        })
                        .or_insert(vals);
                }
            }

            for (_, results) in actual_props_with_temp.iter_mut() {
                results.iter_mut().for_each(|r| {
                    r.meta.mark_for_essence_only = true;
                });
            }

            propagation_result.extend(actual_props_with_temp);
        }

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, _provider: &ItemInfoProvider) -> bool {
        match item.snapshot.rarity {
            // handle special case of tempswap here
            ItemRarityEnum::Rare => !item.helper.has_essences_target.is_empty(),
            _ => false,
        }
    }
}
