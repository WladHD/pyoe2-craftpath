use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
            AffixTierLevelBoundsEnum, ItemRarityEnum, THashMap, THashSet,
        },
    },
    utils::fraction_utils::Fraction,
};

pub struct NormalEssencePropagator;

impl MatrixPropagator for NormalEssencePropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target_item: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        let target_item_affixes = &target_item.affixes;

        let mut pool = provider
            .lookup_base_item_mods(&item_instance.snapshot.base_id)?
            .clone();

        pool.retain(|affix_id, tier_level_holder| {
            let Ok(affix_def) = provider.lookup_affix_definition(&affix_id) else {
                return false;
            };

            // check base and essence affixes
            match affix_def.affix_class {
                AffixClassEnum::Base | AffixClassEnum::Essence => {}
                _ => return false,
            }

            match affix_def.affix_location {
                AffixLocationEnum::Prefix | AffixLocationEnum::Suffix => {}
                _ => return false,
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

            tier_level_holder.retain(|_, tier_level_meta| {
                item_instance.snapshot.item_level >= tier_level_meta.min_item_level
            });

            !tier_level_holder.is_empty()
        });

        for next_affix_of_interest in target_item_affixes
            .iter()
            // TEST IF NEXT AFFIX CAN BE REACHED, SHOULD BE IN POOL OF AVAILABLE MODS
            .filter(|test: &&AffixSpecifier| pool.contains_key(&test.affix))
        {
            let Ok(applicable_essence_ids) = provider.lookup_affix_essences(
                next_affix_of_interest.affix.clone(),
                target_item.base_id.clone(),
            ) else {
                // this affix can not be reached with an essence
                continue;
            };

            for applicable_essence_id in applicable_essence_ids {
                let Ok((essence_def, _, essence_tier)) = provider.collect_essence_info_for_affix(
                    &applicable_essence_id,
                    &target_item.base_id,
                    &next_affix_of_interest.affix,
                ) else {
                    continue;
                };

                if essence_def.name_essence.starts_with("Perfect") {
                    continue;
                }

                // check for all accepted tiers that an essence can achieve
                match next_affix_of_interest.tier.bounds {
                    AffixTierLevelBoundsEnum::Exact
                        if &next_affix_of_interest.tier.tier != essence_tier.0 =>
                    {
                        continue;
                    }
                    AffixTierLevelBoundsEnum::Minimum
                        if &next_affix_of_interest.tier.tier < essence_tier.0 =>
                    {
                        continue;
                    }
                    _ => {}
                };

                let mut affixes = item_instance.snapshot.affixes.clone();
                affixes.insert(AffixSpecifier {
                    affix: next_affix_of_interest.affix.clone(),
                    fractured: false,
                    tier: AffixTierConstraints {
                        bounds: AffixTierLevelBoundsEnum::Exact,
                        tier: essence_tier.0.clone(),
                    },
                });

                let next_item_snapshot = ItemSnapshot {
                    rarity: ItemRarityEnum::Rare,
                    base_id: item_instance.snapshot.base_id.clone(),
                    item_level: item_instance.snapshot.item_level.clone(),
                    affixes: affixes,
                    allowed_sockets: item_instance.snapshot.allowed_sockets.clone(),
                    corrupted: item_instance.snapshot.corrupted.clone(),
                    sockets: item_instance.snapshot.sockets.clone(),
                };

                let hit_chance_fraction = Fraction::one();

                let next_item = PropagationTarget::new(hit_chance_fraction, next_item_snapshot);
                let next_items: Vec<PropagationTarget> = vec![next_item];
                let mut unique_currency_list = CraftCurrencyList {
                    list: THashSet::default(),
                };

                unique_currency_list
                    .list
                    .insert(CraftCurrencyEnum::Essence(applicable_essence_id.clone()));

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
