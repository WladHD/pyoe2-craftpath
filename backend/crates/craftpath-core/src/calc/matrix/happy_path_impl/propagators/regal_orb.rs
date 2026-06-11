use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierLevel,
            AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap, THashSet,
        },
    },
    utils::fraction_utils::Fraction,
};

static REGAL_ORBS: &[CraftCurrencyEnum] = &[
    CraftCurrencyEnum::RegalOrbNormal(),
    CraftCurrencyEnum::RegalOrbGreater(),
    CraftCurrencyEnum::RegalOrbPerfect(),
];

static HOMOGEN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] =
    // &[Some(CraftCurrencyEnum::HomogenisingCoronation()), None];
    &[None];

pub struct RegalOrbPropagator;

impl MatrixPropagator for RegalOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let target_item_affixes = &target_item.affixes;

        let base_group_id = provider.lookup_base_group(&item_instance.snapshot.base_id)?;
        let base_group_def = provider.lookup_base_group_definition(&base_group_id)?;
        let max_affixes_per_side = base_group_def.max_affix / 2;

        for currency in REGAL_ORBS {
            let force_min_starting_level = ItemLevel::from(match currency {
                &CraftCurrencyEnum::RegalOrbNormal() => 0,
                &CraftCurrencyEnum::RegalOrbGreater() => 35,
                &CraftCurrencyEnum::RegalOrbPerfect() => 50,
                _ => continue,
            });

            if force_min_starting_level > item_instance.snapshot.item_level {
                continue;
            }

            for homogen in HOMOGEN_OMEN_GROUP {
                // first check if applying homogen makes sense (0 modgroups = cant)
                match homogen {
                    None => {}
                    Some(_) => {
                        if item_instance.helper.homogenized_mods.is_empty() {
                            continue;
                        }
                    }
                }

                let mut pool = provider
                    .lookup_base_item_mods(&item_instance.snapshot.base_id)?
                    .clone();

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

                    if !blocked_mod_groups_affix
                        .is_disjoint(&item_instance.helper.blocked_modgroups)
                    {
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

                // here would be some follow up pool filtering
                let mut unique_currency_list = CraftCurrencyList {
                    list: THashSet::default(),
                };
                unique_currency_list.list.insert(currency.clone());

                // add homogen omen if exists to combination
                match homogen {
                    None => {}
                    Some(homogen) => {
                        unique_currency_list.list.insert(homogen.clone());
                    }
                }

                let mut next_items: Vec<PropagationTarget> = Vec::new();

                // calc weight for available pool
                let max_weight = pool.iter().fold(0u32, |a, (_, tier_meta)| {
                    a + tier_meta
                        .iter()
                        .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone())
                });

                for next_affix_of_interest in target_item_affixes
                    .iter()
                    // TEST IF NEXT AFFIX CAN BE REACHED, SHOULD BE IN POOL OF AVAILABLE MODS
                    .filter(|test: &&AffixSpecifier| pool.contains_key(&test.affix))
                {
                    let mut affixes: THashSet<AffixSpecifier> =
                        item_instance.snapshot.affixes.clone();
                    affixes.insert(next_affix_of_interest.clone());

                    let next_item_snapshot = ItemSnapshot {
                        rarity: ItemRarityEnum::Rare,
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

                    let affix_chance: u32 = match next_affix_of_interest.tier.bounds {
                        AffixTierLevelBoundsEnum::Minimum => chance_weight
                            .iter()
                            .filter(|(test_tier_level, _provider)| {
                                **test_tier_level <= next_affix_of_interest.tier.tier
                            })
                            .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone()),
                        AffixTierLevelBoundsEnum::Exact => {
                            let Some(cw) = chance_weight.get(&next_affix_of_interest.tier.tier)
                            else {
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

                    let hit_chance_fraction = Fraction::new(affix_chance, max_weight);

                    let next_item = PropagationTarget::new(hit_chance_fraction, next_item_snapshot);

                    next_items.push(next_item);
                }

                propagation_result.insert(unique_currency_list, next_items);
            }
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

        if !base_group_def.is_rare {
            return false;
        }

        match item.snapshot.rarity {
            ItemRarityEnum::Magic => true,
            _ => false,
        }
    }
}
