use std::hash::{Hash, Hasher};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::{
    api::{
        calculator::Calculator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixClassEnum, AffixDefinition, AffixLocationEnum, AffixSpecifier,
            AffixTierLevelBoundsEnum, BaseItemId, ItemLevel, ItemRarityEnum, THashSet,
        },
    },
    utils::{hash_utils::hash_set_unordered, pretty_print_unique_utils::print_affix},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub struct ItemSnapshot {
    pub item_level: ItemLevel,
    pub rarity: ItemRarityEnum,
    pub base_id: BaseItemId,
    pub affixes: THashSet<AffixSpecifier>,
    pub corrupted: bool,
    pub allowed_sockets: u8,
    pub sockets: THashSet<AffixSpecifier>,
}

impl ItemSnapshot {
    pub fn to_pretty_string(
        &self,
        item_provider: &ItemInfoProvider,
        print_affixes: bool,
    ) -> String {
        let mut out = String::new();

        let base_group_id = item_provider.lookup_base_group(&self.base_id).unwrap();
        let base_group_def = item_provider
            .lookup_base_group_definition(&base_group_id)
            .unwrap();

        writeln!(
            &mut out,
            "Base Group: {} (#{}), Max Rarity: {}, Max Affixes: {} ({} per side), Max. Sockets: {} ({} corrupt)",
            base_group_def.name_base_group,
            base_group_id.get_raw_value(),
            match base_group_def.is_rare {
                true => "Rare",
                false => "Magic",
            },
            base_group_def.max_affix,
            base_group_def.max_affix / 2,
            base_group_def.max_sockets,
            base_group_def.max_sockets + 1,
        )
        .unwrap();

        writeln!(
            &mut out,
            "BaseId: #{}, Rarity: {:?}, ItemLevel: {}, Sockets: {}",
            self.base_id.get_raw_value(),
            self.rarity,
            self.item_level.get_raw_value(),
            self.allowed_sockets
        )
        .unwrap();

        if print_affixes {
            for affix in &self.affixes {
                print_affix(
                    &mut out,
                    None,
                    affix,
                    None,
                    &item_provider,
                    false,
                    &self.base_id,
                    false,
                );
            }
        }

        return out;
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl ItemSnapshot {
    #[pyo3(name = "to_pretty_string")]
    pub fn to_pretty_string_py(
        &self,
        item_provider: &ItemInfoProvider,
        print_affixes: bool,
    ) -> String {
        self.to_pretty_string(item_provider, print_affixes)
    }
}

impl Hash for ItemSnapshot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item_level.hash(state);
        self.rarity.hash(state);
        self.base_id.hash(state);
        self.corrupted.hash(state);
        self.allowed_sockets.hash(state);

        let affix_hash = hash_set_unordered(&self.affixes);
        let socket_hash = hash_set_unordered(&self.sockets);

        affix_hash.hash(state);
        socket_hash.hash(state);
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(ItemSnapshot);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct ItemSnapshotHelper {
    // distance of affixes to target
    // 0 -> target item
    // 6 -> empty item, to target item with 6 wanted affixes
    // 12 -> 6 unwanted affixes, to target item with 6 wanted affixes
    pub target_proximity: u8,
    pub prefix_count: u8,
    pub suffix_count: u8,
    pub blocked_modgroups: THashSet<String>,
    pub homogenized_mods: THashSet<u8>,
    pub unwanted_affixes: THashSet<AffixSpecifier>,
    pub is_desecrated: bool,
    pub has_desecrated_target: Option<AffixSpecifier>,
    pub marked_by_abyssal_lord: Option<AffixSpecifier>,
    pub has_essences_target: THashSet<AffixSpecifier>,
}

// idk if item needs to be marked for sth
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct ItemTechnicalMeta {
    pub mark_for_essence_only: bool,
}

impl ItemTechnicalMeta {
    pub fn default() -> Self {
        Self {
            mark_for_essence_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct Item {
    pub snapshot: ItemSnapshot,
    pub helper: ItemSnapshotHelper,
    pub meta: ItemTechnicalMeta,
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(Item, ItemTechnicalMeta, ItemSnapshotHelper);

impl Item {
    pub fn build_with(
        snapshot: ItemSnapshot,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<Self> {
        let mut blocked_modgroups = THashSet::default();
        let mut homogenized_mods = THashSet::default();
        let mut unwanted_affixes = THashSet::default();
        let mut is_desecrated = false;
        let mut prefix_count = 0;
        let mut suffix_count = 0;

        for specifier in &snapshot.affixes {
            let def = provider.lookup_affix_definition(&specifier.affix)?;

            blocked_modgroups.extend(def.exlusive_groups.iter().cloned());
            homogenized_mods.extend(def.tags.iter().cloned());

            if !provider.is_abyssal_mark(&specifier.affix)
                && def.affix_class == AffixClassEnum::Desecrated
            {
                is_desecrated = true;
            }

            // Count by affix location
            match def.affix_location {
                AffixLocationEnum::Prefix => prefix_count += 1,
                AffixLocationEnum::Suffix => suffix_count += 1,
                AffixLocationEnum::Socket => {} // TODO? <-- this will be in own
            }

            // Determine if this affix is unwanted
            let unwanted = match target.affixes.iter().find(|t| t.affix == specifier.affix) {
                Some(t) => match t.tier.bounds {
                    AffixTierLevelBoundsEnum::Exact if t.tier.tier != specifier.tier.tier => true,
                    AffixTierLevelBoundsEnum::Minimum if t.tier.tier < specifier.tier.tier => true,
                    _ => false,
                },
                None => true,
            };

            if unwanted {
                unwanted_affixes.insert(specifier.clone());
            }
        }

        fn find_target<F>(
            target: &THashSet<AffixSpecifier>,
            provider: &ItemInfoProvider,
            pred: F,
        ) -> Option<AffixSpecifier>
        where
            F: Fn(Option<&AffixDefinition>, &AffixSpecifier) -> bool,
        {
            for spec in target.iter() {
                if pred(provider.lookup_affix_definition(&spec.affix).ok(), spec) {
                    return Some(spec.clone());
                }
            }
            None
        }

        let has_desecrated_target = find_target(
            &target.affixes,
            provider,
            |def, _| matches!(def, Some(def) if def.affix_class == AffixClassEnum::Desecrated),
        );

        let marked_by_abyssal_lord = find_target(&target.affixes, provider, |_, spec| {
            provider.is_abyssal_mark(&spec.affix)
        });

        let has_essences_target = target
            .affixes
            .iter()
            .filter_map(|spec| match provider.lookup_affix_definition(&spec.affix) {
                Ok(def) if def.affix_class == AffixClassEnum::Essence => Some(Ok(spec.clone())),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<THashSet<_>, _>>()?;

        let target_proximity =
            Calculator::calculate_target_proximity(&snapshot, &target, &provider)?;

        Ok(Self {
            snapshot,
            helper: ItemSnapshotHelper {
                prefix_count,
                suffix_count,
                blocked_modgroups,
                homogenized_mods,
                unwanted_affixes,
                is_desecrated,
                has_desecrated_target,
                marked_by_abyssal_lord,
                has_essences_target,
                target_proximity,
            },
            meta: ItemTechnicalMeta::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing::instrument;

    use crate::{
        api::{
            item::{Item, ItemSnapshot},
            types::{
                AffixId, AffixSpecifier, AffixTierConstraints, AffixTierLevel,
                AffixTierLevelBoundsEnum, BaseItemId, ItemLevel, ItemRarityEnum, THashMap,
                THashSet,
            },
        },
        external_api::{
            coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider,
            fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order,
        },
        utils::logger_utils::init_tracing,
    };

    #[test]
    #[instrument]
    fn test_item_snapshot() -> Result<()> {
        init_tracing();
        tracing::info!("Checking correct function of ItemSnapshot comparisons");

        let mut item_snapshot_a = ItemSnapshot {
            item_level: ItemLevel::from(100),
            affixes: THashSet::default(),
            base_id: BaseItemId::from(20),
            rarity: ItemRarityEnum::Rare,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        let mut item_snapshot_b = ItemSnapshot {
            item_level: ItemLevel::from(100),
            affixes: THashSet::default(),
            base_id: BaseItemId::from(20),
            rarity: ItemRarityEnum::Rare,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        };

        item_snapshot_a.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5119),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        item_snapshot_b.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5119),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        assert_eq!(item_snapshot_a, item_snapshot_b);
        assert_eq!(item_snapshot_b, item_snapshot_a);

        item_snapshot_a.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5121),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        item_snapshot_b.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5127),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        assert_ne!(item_snapshot_a, item_snapshot_b);

        item_snapshot_b.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5121),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        item_snapshot_a.affixes.insert(AffixSpecifier {
            affix: AffixId::from(5127),
            tier: AffixTierConstraints {
                bounds: AffixTierLevelBoundsEnum::Exact,
                tier: AffixTierLevel::from(3),
            },
            fractured: false,
        });

        assert_eq!(item_snapshot_a, item_snapshot_b);

        tracing::info!("Checking correct function of initializing actual items");

        let hm = THashMap::from_iter(
            vec![(
                "./cache/coe2.json".to_string(),
                "https://www.craftofexile.com/json/poe2/main/poec_data.json".to_string(),
            )]
            .into_iter(),
        );

        let provider = retrieve_contents_from_urls_with_cache_unstable_order(hm, 60_u64 * 60_u64)?;
        let provider = CraftOfExileItemInfoProvider::parse_from_json(
            provider.first().expect("Provider returned no item info"),
        )?;

        let item = Item::build_with(item_snapshot_a.clone(), &item_snapshot_b, &provider)?;
        assert_eq!(item.helper.unwanted_affixes.len(), 0);
        assert_eq!(item.helper.target_proximity, 0); // item reached wanted form

        tracing::info!("{:?}", item);

        item_snapshot_b
            .affixes
            .retain(|test| test.affix != AffixId::from(5119));

        let item = Item::build_with(item_snapshot_a.clone(), &item_snapshot_b, &provider)?;
        assert_eq!(item.helper.unwanted_affixes.len(), 1); // starting item has affix that is not in target
        assert_eq!(item.helper.target_proximity, 2); // item requires removal of affix + applience of affix = 2

        let item = Item::build_with(item_snapshot_b.clone(), &item_snapshot_a, &provider)?;
        assert_eq!(item.helper.unwanted_affixes.len(), 0); // starting items affixes all are included in target 
        assert_eq!(item.helper.target_proximity, 1); // item requires addition of affix = 1

        tracing::info!("{:?}", item);

        Ok(())
    }
}
