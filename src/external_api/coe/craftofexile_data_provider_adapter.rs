use anyhow::{Result, anyhow};

use crate::{
    api::{
        provider::item_info::{AffixWeightTable, ItemInfoProvider},
        types::{
            AffixClassEnum, AffixDefinition, AffixId, AffixTierLevel, AffixTierLevelMeta,
            BaseItemId, EssenceDefinition, EssenceId, EssenceTierLevelMeta, ItemLevel, THashMap,
            THashSet, Weight,
        },
    },
    external_api::coe::craftofexile_json_definition::CoEGameData,
};

#[derive(Debug)]
pub struct ItemDataProviderCache {
    pub base_item_affix_weight_table: THashMap<BaseItemId, AffixWeightTable>,
    pub affix_definition_table: THashMap<AffixId, AffixDefinition>,
    pub affix_essence_table: THashMap<AffixId, EssenceId>,
    pub essence_definition_table: THashMap<EssenceId, EssenceDefinition>,
}

pub struct CraftOfExileItemInfoProvider;

impl CraftOfExileItemInfoProvider {
    pub fn parse_from_json(mut text: &str) -> Result<ItemInfoProvider> {
        // coe's data starts with that, needs to be cleaned first
        if text.starts_with("poecd=") {
            text = &text["poecd=".len()..];
        }

        let parsed: CoEGameData = serde_json::from_str(&text)
            .map_err(|err| anyhow!("Could not parse provided game items. \nERROR: {:?}", err))?;

        let mut transformed_cache = ItemDataProviderCache {
            affix_definition_table: THashMap::default(),
            base_item_affix_weight_table: THashMap::default(),
            affix_essence_table: THashMap::default(),
            essence_definition_table: THashMap::default(),
        };

        for instantiated_item in parsed.bitems.seq.iter() {
            let base_item_id = BaseItemId::from(instantiated_item.id_base);

            if transformed_cache
                .base_item_affix_weight_table
                .contains_key(&base_item_id)
            {
                continue;
            }

            let mut item_affix_map: AffixWeightTable = THashMap::default();

            // get possible mods for item
            let raw_affixes_for_a_base_item = parsed.basemods.get(&*base_item_id.get_raw_value());

            let raw_affixes_for_a_base_item = match raw_affixes_for_a_base_item {
                Some(e) => e,
                None => {
                    tracing::warn!(
                        "Skipping item base '{}' because it had no defined base mods.",
                        base_item_id.get_raw_value()
                    );
                    continue;
                }
            };

            // iterate over possible mods for an item and parse weights
            for raw_base_mod in raw_affixes_for_a_base_item {
                let affix_id = AffixId::from(raw_base_mod.clone());

                // ## METHOD 1
                // calculate affix weight for current item
                let tier = parsed
                    .tiers
                    .iter()
                    .find(|(raw_affix_id, _)| **raw_affix_id == *affix_id.get_raw_value())
                    .and_then(|(_, tier_list)| {
                        tier_list.iter().find_map(|(raw_item_base_id, tiers)| {
                            if raw_item_base_id == base_item_id.get_raw_value() {
                                Some(tiers)
                            } else {
                                None
                            }
                        })
                    });

                match tier {
                    None => {
                        tracing::warn!(
                            "Could not find tiers for affix {:?} and item base {:?}",
                            affix_id.get_raw_value(),
                            base_item_id.get_raw_value()
                        )
                    }
                    Some(e) => {
                        let mut item_affix_weight: THashMap<AffixTierLevel, AffixTierLevelMeta> =
                            THashMap::default();

                        let tier_amount = e.len();

                        // TODO: alg relies on position in Vec -> susceptible for errors on change
                        e.iter().enumerate().for_each(|(index, tier)| {
                            item_affix_weight.insert(
                                AffixTierLevel::from((tier_amount - index) as u8),
                                AffixTierLevelMeta {
                                    min_item_level: ItemLevel::from(tier.ilvl),
                                    weight: Weight::from(tier.weighting),
                                },
                            );
                        });

                        item_affix_map.insert(affix_id.clone(), item_affix_weight);
                    }
                }

                // ## METHOD 2
                // build only affixes that are referenced by items .. should be the case anyway but w/e
                if transformed_cache
                    .affix_definition_table
                    .contains_key(&affix_id)
                {
                    continue;
                }

                let affix_info = parsed
                    .modifiers
                    .seq
                    .iter()
                    .find(|test| test.id_modifier == *affix_id.get_raw_value());

                let affix_info = match affix_info {
                    Some(e) => e,
                    None => {
                        tracing::warn!(
                            "Skipping affix '{}' for item '{}' because the affix had no corresponding meta information.",
                            affix_id.get_raw_value(),
                            base_item_id.get_raw_value()
                        );
                        continue;
                    }
                };

                let mut affix_def = AffixDefinition {
                    exlusive_groups: THashSet::default(),
                    description_template: affix_info.name_modifier.clone(),
                    affix_class: affix_info.id_mgroup.clone(),
                    tags: THashSet::default(),
                    affix_location: affix_info.affix.clone(),
                };

                affix_def
                    .exlusive_groups
                    .extend(affix_info.modgroups.clone());

                affix_def.tags.extend(affix_info.mtypes.clone());

                transformed_cache
                    .affix_definition_table
                    .insert(affix_id, affix_def);

                // ESSENCE CHECK

                match affix_info.id_mgroup {
                    AffixClassEnum::Essence => {
                        // this assumes that the essences mapping are the same for every item
                        // which would make sense. if not this could be a bug ... simplifies
                        // mapping so will leave it for now until proven otherwise
                        parsed
                            .essences
                            .dir
                            .get(base_item_id.get_raw_value())
                            .iter()
                            .for_each(|raw_affix_essence_table| {
                                raw_affix_essence_table.iter().for_each(|(a, e)| {
                                    transformed_cache
                                        .affix_essence_table
                                        .insert(AffixId::from(*a), EssenceId::from(*e));
                                })
                            });
                    }
                    _ => {}
                }
            }

            transformed_cache
                .base_item_affix_weight_table
                .insert(base_item_id.clone(), item_affix_map);
        }

        // insert essence meta
        transformed_cache
            .affix_essence_table
            .iter()
            .for_each(|(_affix_id, essence_id)| {
                if let Some(essence) = parsed
                    .essences
                    .seq
                    .iter()
                    .find(|test| test.id_essence == *essence_id.get_raw_value())
                {
                    let mut essence_tiers: THashMap<
                        BaseItemId,
                        THashMap<AffixId, EssenceTierLevelMeta>,
                    > = THashMap::default();

                    essence.tiers.iter().for_each(|(raw_base, raw_tiers)| {
                        let base_id = BaseItemId::from(*raw_base);
                        let mut hm: THashMap<AffixId, EssenceTierLevelMeta> = THashMap::default();

                        raw_tiers.iter().for_each(|e| {
                            e.iter().for_each(|e| {
                                hm.insert(
                                    AffixId::from(e.r#mod),
                                    EssenceTierLevelMeta {
                                        id: e.id.clone(),
                                        min_item_level: ItemLevel::from(e.ilvl),
                                    },
                                );
                            })
                        });

                        essence_tiers.insert(base_id, hm);
                    });

                    transformed_cache.essence_definition_table.insert(
                        essence_id.clone(),
                        EssenceDefinition {
                            corrupt: essence.corrupt,
                            name_essence: essence.name_essence.clone(),
                            base_tier_table: essence_tiers,
                        },
                    );
                }
            });

        Ok(ItemInfoProvider::new(
            transformed_cache.affix_definition_table,
            transformed_cache.base_item_affix_weight_table,
            transformed_cache.affix_essence_table,
            transformed_cache.essence_definition_table,
        ))
    }
}
