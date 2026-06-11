use anyhow::{Result, anyhow};

use crate::{
    api::{
        provider::item_info::{AffixWeightTable, ItemInfoProvider},
        types::{
            AffixDefinition, AffixId, AffixTierLevel, AffixTierLevelMeta, BaseGroupDefinition,
            EssenceKindEnum,
            BaseGroupId, BaseItemId, EssenceDefinition, EssenceId, EssenceTierLevelMeta, ItemLevel,
            THashMap, THashSet, Weight,
        },
    },
    external_api::coe::craftofexile_json_definition::CoEGameData,
};

#[derive(Debug)]
struct ItemDataProviderCache {
    pub base_item_affix_weight_table: THashMap<BaseItemId, AffixWeightTable>,
    pub affix_definition_table: THashMap<AffixId, AffixDefinition>,
    pub affix_essence_table: THashMap<(AffixId, BaseItemId), THashSet<EssenceId>>,
    pub essence_definition_table: THashMap<EssenceId, EssenceDefinition>,
    pub base_group_mappings: THashMap<BaseItemId, BaseGroupId>,
    pub base_group_definition: THashMap<BaseGroupId, BaseGroupDefinition>,
}

#[derive(Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub struct CraftOfExileItemInfoProvider;

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl CraftOfExileItemInfoProvider {
    #[pyo3(name = "parse_from_json")]
    #[staticmethod]
    pub fn parse_from_json_py(text: &str) -> pyo3::PyResult<ItemInfoProvider> {
        Self::parse_from_json(text)
            .map_err(crate::utils::py_err_utils::to_py_err)
    }
}

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
            base_group_mappings: THashMap::default(),
            base_group_definition: THashMap::default(),
        };

        for base in parsed.bases.seq.iter() {
            transformed_cache.base_group_mappings.insert(
                BaseItemId::from(base.id_base),
                BaseGroupId::from(base.id_bgroup),
            );
        }

        for raw_base_group in parsed.bgroups.seq.iter() {
            let base_definition = BaseGroupDefinition {
                max_affix: raw_base_group.max_affix,
                max_sockets: raw_base_group.max_sockets,
                name_base_group: raw_base_group.name_bgroup.clone(),
                is_rare: raw_base_group.is_rare,
            };

            transformed_cache
                .base_group_definition
                .insert(BaseGroupId::from(raw_base_group.id_bgroup), base_definition);
        }

        for raw_base in parsed.bases.seq.iter() {
            let base_item_id = BaseItemId::from(raw_base.id_base);

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
            // nvm: BaseMod and possible tiers are not interchangable, just find out current bid
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

                let affix_location = match &affix_info.affix {
                    Some(location) => location.clone(),
                    None => {
                        tracing::debug!(
                            "Skipping affix '{}' for item '{}' because its affix type is unknown.",
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
                    affix_location,
                };

                affix_def
                    .exlusive_groups
                    .extend(affix_info.modgroups.clone());

                affix_def.tags.extend(affix_info.mtypes.clone());

                transformed_cache
                    .affix_definition_table
                    .insert(affix_id, affix_def);
            }

            transformed_cache
                .base_item_affix_weight_table
                .insert(base_item_id.clone(), item_affix_map);
        }

        // ESSENCE CHECK
        // TODO: Currently this method fetches only the lowest essences
        // or perfect essences. If for an affix multiple essences can be used, to specify different tiers
        // only one is taken. This needs to be reworked and mapped from essence mods directly.
        parsed.essences.seq.iter().for_each(|essence| {
            let essence_id = EssenceId::from(essence.id_essence);
            let mut essence_tiers: THashMap<BaseItemId, THashMap<AffixId, EssenceTierLevelMeta>> =
                THashMap::default();

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

                        let key = (AffixId::from(e.r#mod), base_id.clone());

                        transformed_cache
                            .affix_essence_table
                            .entry(key.clone())
                            .or_default()
                            .insert(essence_id.clone());
                    })
                });

                essence_tiers.insert(base_id, hm);
            });

            // classify by name: the data has no explicit kind marker
            let kind = if essence.name_essence.starts_with("Perfect") {
                EssenceKindEnum::Perfect
            } else if essence.name_essence.contains("Alloy") {
                EssenceKindEnum::Alloy
            } else {
                EssenceKindEnum::Standard
            };

            transformed_cache.essence_definition_table.insert(
                essence_id.clone(),
                EssenceDefinition {
                    corrupt: essence.corrupt,
                    name_essence: essence.name_essence.clone(),
                    base_tier_table: essence_tiers,
                    kind,
                },
            );
        });

        Ok(ItemInfoProvider {
            cache_affix_def: transformed_cache.affix_definition_table,
            cache_item_affix_table: transformed_cache.base_item_affix_weight_table,
            cache_affix_essence_table: transformed_cache.affix_essence_table,
            cache_essence_def: transformed_cache.essence_definition_table,
            cache_base_group_table: transformed_cache.base_group_mappings,
            base_group_definition: transformed_cache.base_group_definition,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::domain::item::{Item, ItemSnapshot};
    use crate::domain::types::{
        AffixLocationEnum, AffixSpecifier, AffixTierConstraints, AffixTierLevel,
        AffixTierLevelBoundsEnum, ItemLevel, ItemRarityEnum, THashMap, THashSet,
    };
    use crate::features::data::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
    use crate::features::data::http::retrieve_contents_from_urls_with_cache_unstable_order;

    /// Game patch 0.5.0: the CoE data carries corruption-implicit modifiers
    /// (affix type "corrupted"). They must parse into the provider with the
    /// Corrupted location and never count toward prefix/suffix capacity.
    #[test]
    fn test_corrupted_implicits_parse_and_do_not_consume_affix_slots() -> Result<()> {
        let hm = THashMap::from_iter(vec![(
            "./cache/coe2.json".to_string(),
            "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
        )]);
        let jsons = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60 * 60 * 24)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(jsons.first().unwrap())?;

        // corrupted definitions parsed (CoE ships ~101 of them, referenced by
        // most bases; the exact count may drift with patches)
        let corrupted: Vec<_> = provider
            .cache_affix_def
            .iter()
            .filter(|(_, def)| def.affix_location == AffixLocationEnum::Corrupted)
            .collect();
        assert!(
            corrupted.len() > 50,
            "expected >50 corrupted implicit definitions, got {}",
            corrupted.len()
        );

        // find a base whose weight table references a corrupted mod so the
        // snapshot below is consistent with real data
        let (base_id, corrupted_affix_id) = provider
            .cache_item_affix_table
            .iter()
            .find_map(|(base_id, table)| {
                table.keys().find_map(|affix_id| {
                    let def = provider.lookup_affix_definition(affix_id).ok()?;
                    (def.affix_location == AffixLocationEnum::Corrupted)
                        .then(|| (base_id.clone(), affix_id.clone()))
                })
            })
            .expect("no base references a corrupted implicit");

        let mut affixes: THashSet<AffixSpecifier> = THashSet::default();
        affixes.insert(AffixSpecifier {
            affix: corrupted_affix_id,
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(1),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        });

        let snapshot = ItemSnapshot {
            item_level: ItemLevel::from(81),
            rarity: ItemRarityEnum::Rare,
            base_id,
            affixes,
            corrupted: true,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let item = Item::build_with(snapshot.clone(), &snapshot, &provider)?;
        assert_eq!(item.helper.prefix_count, 0);
        assert_eq!(item.helper.suffix_count, 0);

        Ok(())
    }
}
