use anyhow::{Result, anyhow};

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
            AffixTierLevel, AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap,
            THashSet,
        },
    },
    utils::fraction_utils::Fraction,
};

static EXALTED_ORBS: &[CraftCurrencyEnum] = &[
    CraftCurrencyEnum::ExaltedOrbNormal(),
    CraftCurrencyEnum::ExaltedOrbGreater(),
    CraftCurrencyEnum::ExaltedOrbPerfect(),
];

static DEX_SIN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::SinistralExaltation()),
    Some(CraftCurrencyEnum::DextralExaltation()),
    None,
];

static HOMOGEN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] =
    // &[Some(CraftCurrencyEnum::HomogenisingExaltation()), None];
    &[None];

pub struct ExaltedOrbPropagator;

impl ExaltedOrbPropagator {
    pub fn propagate_step_explicit(
        currency: &CraftCurrencyEnum,
        dex_sin: Option<&CraftCurrencyEnum>,
        homogen: Option<&CraftCurrencyEnum>,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        force_unwanted_location: Option<&AffixLocationEnum>,
        provider: &ItemInfoProvider,
    ) -> Result<Vec<PropagationTarget>> {
        let target_affixes = &target_item.affixes;

        let force_min_starting_level = ItemLevel::from(match currency {
            &CraftCurrencyEnum::ExaltedOrbNormal() => 0,
            &CraftCurrencyEnum::ExaltedOrbGreater() => 35,
            &CraftCurrencyEnum::ExaltedOrbPerfect() => 50,
            _ => {
                return Err(anyhow!("Unknown currency"));
            }
        });

        if force_min_starting_level > item_instance.snapshot.item_level {
            return Ok(Vec::new());
        }

        // TODO check if applying homogen actually constraints homogen groups
        // first check if applying homogen makes sense (0 modgroups = cant)
        match homogen {
            None => {}
            Some(_) => {
                if item_instance.helper.homogenized_mods.is_empty() {
                    return Ok(Vec::new());
                }
            }
        }

        let mut pool = provider
            .lookup_base_item_mods(&item_instance.snapshot.base_id)?
            .clone();

        let base_group_id = provider.lookup_base_group(&item_instance.snapshot.base_id)?;
        let base_group_def = provider.lookup_base_group_definition(&base_group_id)?;
        let max_affixes_per_side = base_group_def.max_affix / 2;

        // since we start with normal item, not all checks are needed (cauz no blocking groups etc.)
        pool.retain(|affix_id, tier_level_holder| {
            let Ok(affix_def) = provider.lookup_affix_definition(&affix_id) else {
                return false;
            };

            // check if affix can be applied with homogen restriction
            match homogen {
                None => {}
                Some(_) => {
                    if affix_def
                        .tags
                        .is_disjoint(&item_instance.helper.homogenized_mods)
                    {
                        return false;
                    }
                }
            }

            // check if dex sin omen is applied
            match dex_sin {
                None => {}
                Some(dex_sin) => {
                    match dex_sin {
                        CraftCurrencyEnum::DextralExaltation()
                            if (affix_def.affix_location != AffixLocationEnum::Suffix) =>
                        {
                            return false;
                        }

                        CraftCurrencyEnum::SinistralExaltation()
                            if (affix_def.affix_location != AffixLocationEnum::Prefix) =>
                        {
                            return false;
                        }
                        _ => {}
                    };
                }
            }

            match affix_def.affix_class {
                AffixClassEnum::Base => {}
                _ => return false,
            }

            match affix_def.affix_location {
                AffixLocationEnum::Prefix | AffixLocationEnum::Suffix => {}
                _ => return false,
            }

            // filter out next affixes based on max. suffix / prefix (magic item = max 1. each)
            match affix_def.affix_location {
                AffixLocationEnum::Prefix => {
                    if item_instance.helper.prefix_count >= max_affixes_per_side {
                        return false;
                    }
                }
                AffixLocationEnum::Suffix => {
                    if item_instance.helper.suffix_count >= max_affixes_per_side {
                        return false;
                    }
                }
                _ => return false, // cant be reached with this craft method
            }

            // check if item has same affix already, skip. (even if item tier is not accepted -> intentional)
            if item_instance
                .snapshot
                .affixes
                .iter()
                .any(|item_affixes| &item_affixes.affix == affix_id)
            {
                return false;
            }

            let blocked_mod_groups_affix = &affix_def.exlusive_groups;

            if !blocked_mod_groups_affix.is_disjoint(&item_instance.helper.blocked_modgroups) {
                // should this be handled differently?
                return false;
            }

            // CAREFUL!!! Tier 1 mods CANNOT be excluded even if higher currencies dictate higher minimal item level
            // e. g.
            tier_level_holder.retain(|tier, tier_level_meta| {
                item_instance.snapshot.item_level >= tier_level_meta.min_item_level
                    && (tier_level_meta.min_item_level >= force_min_starting_level
                        || tier == &AffixTierLevel::from(1))
            });

            !tier_level_holder.is_empty()
        });

        let mut next_items: Vec<PropagationTarget> = Vec::new();

        // calc weight for available pool
        let max_weight = pool.iter().fold(0u32, |a, (_, tier_meta)| {
            a + tier_meta
                .iter()
                .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone())
        });

        let mut next_affix_pool: Vec<AffixSpecifier> = match force_unwanted_location {
            None => target_affixes
                .iter()
                // TEST IF NEXT AFFIX CAN BE REACHED, SHOULD BE IN POOL OF AVAILABLE MODS
                .filter(|test: &&AffixSpecifier| pool.contains_key(&test.affix))
                .cloned()
                .collect(),
            //
            Some(loc) => pool
                .iter()
                .filter(|test| {
                    target_affixes.iter().all(|t| &t.affix != test.0)
                        && loc
                            == &provider
                                .lookup_affix_definition(&test.0)
                                .unwrap()
                                .affix_location
                })
                .map(|e| AffixSpecifier {
                    affix: e.0.clone(),
                    tier: AffixTierConstraints {
                        bounds: AffixTierLevelBoundsEnum::Minimum,
                        tier: e.1.keys().max().cloned().unwrap(),
                    },
                    fractured: false,
                })
                .collect(),
        };

        // this is used for perfect essence intermediary step
        let actual_affix_pool: u32 =
            next_affix_pool
                .iter()
                .fold(0_u32, |a, next_affix_of_interest| {
                    let chance_weight = pool.get(&next_affix_of_interest.affix).unwrap();

                    let affix_chance: u32 = match next_affix_of_interest.tier.bounds {
                        AffixTierLevelBoundsEnum::Minimum => chance_weight
                            .iter()
                            .filter(|(test_tier_level, _provider)| {
                                **test_tier_level <= next_affix_of_interest.tier.tier
                            })
                            .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone()),
                        AffixTierLevelBoundsEnum::Exact => {
                            let cw = chance_weight
                                .get(&next_affix_of_interest.tier.tier)
                                .unwrap();
                            cw.weight.get_raw_value().clone()
                        }
                    };

                    a + affix_chance
                });

        if force_unwanted_location.is_some() {
            // TODO: instead of pinning the combined chance against an existing affix,
            // create a new type TempSuffix / TempPrefix to signal, that the actual info
            // about the affix is not needed
            let min = next_affix_pool
                .iter()
                .min_by(|a, b| a.affix.cmp(&b.affix))
                .cloned();

            next_affix_pool.retain(|test| match &min {
                Some(e) => &e.affix == &test.affix,
                None => false,
            });
        }

        for next_affix_of_interest in next_affix_pool.iter() {
            let mut affixes: THashSet<AffixSpecifier> = item_instance.snapshot.affixes.clone();
            affixes.insert(next_affix_of_interest.clone());

            let next_item_snapshot = ItemSnapshot {
                rarity: item_instance.snapshot.rarity.clone(),
                base_id: item_instance.snapshot.base_id.clone(),
                item_level: item_instance.snapshot.item_level.clone(),
                affixes: affixes,
                allowed_sockets: item_instance.snapshot.allowed_sockets.clone(),
                corrupted: item_instance.snapshot.corrupted.clone(),
                sockets: item_instance.snapshot.sockets.clone(),
            };

            let Some(chance_weight) = pool.get(&next_affix_of_interest.affix) else {
                continue;
            };

            let mut affix_chance: u32 = match next_affix_of_interest.tier.bounds {
                AffixTierLevelBoundsEnum::Minimum => chance_weight
                    .iter()
                    .filter(|(test_tier_level, _provider)| {
                        **test_tier_level <= next_affix_of_interest.tier.tier
                    })
                    .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone()),
                AffixTierLevelBoundsEnum::Exact => {
                    let Some(cw) = chance_weight.get(&next_affix_of_interest.tier.tier) else {
                        continue;
                    };

                    cw.weight.get_raw_value().clone()
                }
            };

            // should not happen.               :)
            if affix_chance == 0 {
                tracing::trace!("it happened");
                continue;
            }

            if force_unwanted_location.is_some() {
                affix_chance = actual_affix_pool;
            }

            let hit_chance_fraction = Fraction::new(affix_chance, max_weight);

            let next_item = PropagationTarget::new(hit_chance_fraction, next_item_snapshot);

            next_items.push(next_item);
        }

        Ok(next_items)
    }

    pub fn propagate_step_unwanted_location_bound(
        item_instance: &Item,
        target_item: &ItemSnapshot,
        target_location: &AffixLocationEnum,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let dex_sin = match target_location {
            AffixLocationEnum::Prefix => vec![Some(CraftCurrencyEnum::SinistralExaltation()), None],
            AffixLocationEnum::Suffix => vec![Some(CraftCurrencyEnum::DextralExaltation()), None],
            _ => Vec::new(),
        };

        for dex_sin in dex_sin.iter() {
            let mut next_items = ExaltedOrbPropagator::propagate_step_explicit(
                &CraftCurrencyEnum::ExaltedOrbNormal(),
                dex_sin.as_ref(),
                None,
                &item_instance,
                &target_item,
                Some(target_location),
                &provider,
            )?;

            let mut unique_currency_list = CraftCurrencyList {
                list: THashSet::default(),
            };
            unique_currency_list
                .list
                .insert(CraftCurrencyEnum::ExaltedOrbNormal().clone());

            if let Some(dex_sin) = dex_sin {
                unique_currency_list.list.insert(dex_sin.clone());
            }

            let merged = next_items.drain(..).reduce(|mut acc, b| {
                acc.chance = acc.chance + b.chance;
                acc
            });

            next_items.clear();
            if let Some(m) = merged {
                next_items.push(m);
            }

            propagation_result.insert(unique_currency_list, next_items);
        }

        Ok(propagation_result)
    }

    pub fn propagate_step_default(
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        for currency in EXALTED_ORBS {
            for dex_sin in DEX_SIN_OMEN_GROUP {
                for homogen in HOMOGEN_OMEN_GROUP {
                    let next_items = ExaltedOrbPropagator::propagate_step_explicit(
                        &currency,
                        dex_sin.as_ref(),
                        homogen.as_ref(),
                        &item_instance,
                        &target_item,
                        None,
                        &provider,
                    )?;

                    let mut unique_currency_list = CraftCurrencyList {
                        list: THashSet::default(),
                    };
                    unique_currency_list.list.insert(currency.clone());

                    if let Some(homogen) = homogen {
                        unique_currency_list.list.insert(homogen.clone());
                    }

                    if let Some(dex_sin) = dex_sin {
                        unique_currency_list.list.insert(dex_sin.clone());
                    }

                    propagation_result.insert(unique_currency_list, next_items);
                }
            }
        }

        Ok(propagation_result)
    }
}

// TODO !!!! optimize omen use.. if no change is observed while using it, dont propagate that route
impl MatrixPropagator for ExaltedOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        ExaltedOrbPropagator::propagate_step_default(item_instance, target_item, provider)
    }

    fn is_applicable(&self, item: &Item, _provider: &ItemInfoProvider) -> bool {
        match item.snapshot.rarity {
            ItemRarityEnum::Rare => item.snapshot.affixes.len() < 6,
            _ => false,
        }
    }
}
