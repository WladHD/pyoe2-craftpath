use anyhow::Result;


use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierLevelBoundsEnum,
            ItemRarityEnum, THashMap, THashSet,
        },
    },
    utils::fraction_utils::Fraction,
};

/// Reroll-omen combinations and the resulting number of picks `k` of the
/// desecration 3-option choice (MECHANICS.md V6): baseline = one set of
/// options, Omen of Abyssal Echoes allows rerolling the set once (k=2).
/// (Omen of Light is an ANNULMENT omen - see orb_of_annulment.rs.)
static REROLL_OMEN_COMBOS: &[(&[CraftCurrencyEnum], u32)] = &[
    (&[], 1),
    (&[CraftCurrencyEnum::AbyssalEchoes()], 2),
];

static DEX_SIN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::SinistralNecromancy()),
    Some(CraftCurrencyEnum::DextralNecromancy()),
    None,
];

// DISALLOW "LOOSE" DESECRATION FOR NOW
static HOMOGEN_OMEN_GROUP: &[Option<CraftCurrencyEnum>] = &[
    Some(CraftCurrencyEnum::TheBlackblooded()),
    Some(CraftCurrencyEnum::TheSovereign()),
    Some(CraftCurrencyEnum::TheLiege()),
];

pub struct DesecrationPropagator;

impl DesecrationPropagator {
    pub fn propagate_step_explicit(
        reroll_picks: u32,
        dex_sin: Option<&CraftCurrencyEnum>,
        constraint_modgroup: Option<&CraftCurrencyEnum>,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<Vec<PropagationTarget>> {
        let target_affixes = &target_item.affixes;

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

            // check if affix can be applied with Kurgal, Ulaman restriction
            match constraint_modgroup {
                None => {}
                Some(omen) => match omen {
                    CraftCurrencyEnum::TheBlackblooded() => {
                        if !affix_def.tags.contains(&40u8) {
                            return false;
                        }
                    }
                    CraftCurrencyEnum::TheSovereign() => {
                        if !affix_def.tags.contains(&41u8) {
                            return false;
                        }
                    }
                    CraftCurrencyEnum::TheLiege() => {
                        if !affix_def.tags.contains(&39u8) {
                            return false;
                        }
                    }
                    _ => {}
                },
            }

            // useless for now ... since constraint_modgroup wipes others anyway
            match affix_def.affix_class {
                AffixClassEnum::Desecrated => {}
                _ => return false,
            }

            // check if dex sin omen is applied
            match dex_sin {
                None => {}
                Some(dex_sin) => {
                    match dex_sin {
                        CraftCurrencyEnum::DextralNecromancy()
                            if (affix_def.affix_location != AffixLocationEnum::Suffix) =>
                        {
                            return false;
                        }

                        // NOTE: this previously matched SinistralExaltation()
                        // by mistake, so the prefix filter never fired for
                        // Sinistral Necromancy.
                        CraftCurrencyEnum::SinistralNecromancy()
                            if (affix_def.affix_location != AffixLocationEnum::Prefix) =>
                        {
                            return false;
                        }
                        _ => {}
                    };
                }
            }

            match affix_def.affix_location {
                AffixLocationEnum::Prefix | AffixLocationEnum::Suffix => {}
                _ => return false,
            }

            let will_be_replaced_anyway = |location: AffixLocationEnum| -> bool {
                item_instance
                    .helper
                    .marked_by_abyssal_lord
                    .as_ref()
                    .and_then(|marked| provider.lookup_affix_definition(&marked.affix).ok())
                    .map_or(false, |def| def.affix_location == location)
            };

            // filter out next affixes based on max. suffix / prefix (magic item = max 1. each)
            match affix_def.affix_location {
                AffixLocationEnum::Prefix => {
                    if item_instance.helper.prefix_count >= max_affixes_per_side
                        && !will_be_replaced_anyway(AffixLocationEnum::Prefix)
                    {
                        return false;
                    }
                }
                AffixLocationEnum::Suffix => {
                    if item_instance.helper.suffix_count >= max_affixes_per_side
                        && !will_be_replaced_anyway(AffixLocationEnum::Suffix)
                    {
                        return false;
                    }
                }
                _ => return false, // cant be reached with this craft method
            }

            // this should be useless here
            // (((check if item has same affix already, skip. (even if item tier is not accepted -> intentional))))
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

            // cant be applied on only desecrated mods ...
            // (((CAREFUL!!! Tier 1 mods CANNOT be excluded even if higher currencies dictate higher minimal item level)))
            // e. g.
            // tier_level_holder.retain(|tier, tier_level_meta| {
            //     item_instance.snapshot.item_level >= tier_level_meta.min_item_level
            //         && (tier_level_meta.min_item_level >= force_min_starting_level
            //             || tier == &AffixTierLevel::from(1))
            // });

            !tier_level_holder.is_empty()
        });

        let mut next_items: Vec<PropagationTarget> = Vec::new();

        // calc weight for available pool
        let max_weight = pool.iter().fold(0u32, |a, (_, tier_meta)| {
            a + tier_meta
                .iter()
                .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone())
        });

        for next_affix_of_interest in target_affixes
            .iter()
            // TEST IF NEXT AFFIX CAN BE REACHED, SHOULD BE IN POOL OF AVAILABLE MODS
            .filter(|test: &&AffixSpecifier| pool.contains_key(&test.affix))
        {
            let mut affixes: THashSet<AffixSpecifier> = item_instance.snapshot.affixes.clone();
            affixes.insert(next_affix_of_interest.clone());

            // TODO: this removes "Bears the Mark of the Abyssal Lord" on desecration
            // not tested yet, especially needs to get special-cased later
            affixes.retain(|test| !provider.is_abyssal_mark(&test.affix));

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

            let affix_chance: u32 = match next_affix_of_interest.tier.bounds {
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

            let hit_chance_fraction =
                hit_chance_at_least_once(max_weight, affix_chance, reroll_picks);

            let next_item = PropagationTarget::new(hit_chance_fraction, next_item_snapshot);

            next_items.push(next_item);
        }

        Ok(next_items)
    }
}

// TODO optimize omen use.. if no change is observed while using it, dont propagate that route
impl MatrixPropagator for DesecrationPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        for (reroll_omens, reroll_picks) in REROLL_OMEN_COMBOS {
            for dex_sin in DEX_SIN_OMEN_GROUP {
                for constraint_modgroup in HOMOGEN_OMEN_GROUP {
                    if let Ok(next_items) = DesecrationPropagator::propagate_step_explicit(
                        *reroll_picks,
                        dex_sin.as_ref(),
                        constraint_modgroup.as_ref(),
                        &item_instance,
                        &target_item,
                        &provider,
                    ) {
                        let mut unique_currency_list = CraftCurrencyList {
                            list: THashSet::default(),
                        };

                        for reroll_omen in reroll_omens.iter() {
                            unique_currency_list.list.insert(reroll_omen.clone());
                        }

                        if let Some(modgroups) = constraint_modgroup {
                            unique_currency_list.list.insert(modgroups.clone());
                        }

                        if let Some(dex_sin) = dex_sin {
                            unique_currency_list.list.insert(dex_sin.clone());
                        }

                        // disable "loose" desecration
                        if unique_currency_list.list.is_empty() {
                            continue;
                        }

                        unique_currency_list
                            .list
                            .insert(CraftCurrencyEnum::Desecrator(
                                item_instance.snapshot.base_id.clone(),
                                provider.lookup_base_group(&item_instance.snapshot.base_id)?,
                            ));

                        propagation_result.insert(unique_currency_list, next_items);
                    }
                }
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

        match item.snapshot.rarity {
            ItemRarityEnum::Rare => {
                // TODO disallow random removal when affixes are full
                // implement annuli like special case on 6
                item.helper.has_desecrated_target.is_some()
                    && !item.helper.is_desecrated
                    && (item.snapshot.affixes.len() as u8) < base_group_def.max_affix
            }
            _ => false,
        }
    }
}

/// Compute probability of hitting a target affix at least once
/// N: total affixes
/// n: number of draws per pull
/// k: number of pulls
///
/// Uses the identity C(N-1, n) / C(N, n) = (N - n) / N so no binomials are
/// computed (the previous implementation overflowed u32 for weight-sized N),
/// plus u128 intermediates for the k-th power with a precision-degrading
/// fallback should the reduced fraction still not fit u32.
fn hit_chance_at_least_once(total_amount: u32, n: u32, k: u32) -> Fraction {
    if n >= total_amount {
        return Fraction::new(1, 1); // you always hit if you draw the whole pool
    }

    // miss in a single pull: (N - n) / N; miss in all k pulls: ((N - n) / N)^k
    let mut miss_num: u128 = 1;
    let mut miss_den: u128 = 1;
    for _ in 0..k {
        miss_num *= (total_amount - n) as u128;
        miss_den *= total_amount as u128;
    }

    // hit at least once = 1 - miss_all = (den - num) / den
    let mut hit_num = miss_den - miss_num;
    let mut hit_den = miss_den;

    let g = num_integer::gcd(hit_num, hit_den);
    hit_num /= g;
    hit_den /= g;

    // degrade precision only if u32 cannot hold the reduced fraction
    while hit_num > u32::MAX as u128 || hit_den > u32::MAX as u128 {
        hit_num >>= 1;
        hit_den >>= 1;
    }

    Fraction::new(hit_num.max(1) as u32, hit_den.max(1) as u32)
}

#[cfg(test)]
mod tests {
    use super::hit_chance_at_least_once;

    /// More picks of the desecration choice (Abyssal Echoes k=2, plus Omen of
    /// Light k=3) must strictly increase the hit chance while staying <= 1.
    #[test]
    fn test_hit_chance_monotonic_in_reroll_picks() {
        let total = 100;
        let wanted = 7;

        let k1 = hit_chance_at_least_once(total, wanted, 1).to_f64();
        let k2 = hit_chance_at_least_once(total, wanted, 2).to_f64();
        let k3 = hit_chance_at_least_once(total, wanted, 3).to_f64();

        assert!(k1 > 0.0);
        assert!(k2 > k1, "echoes must improve the chance: {k2} <= {k1}");
        assert!(k3 > k2, "omen of light must improve further: {k3} <= {k2}");
        assert!(k3 <= 1.0);
    }
}
