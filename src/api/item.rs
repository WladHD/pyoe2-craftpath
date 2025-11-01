use std::hash::{Hash, Hasher};

use crate::{
    api::types::{AffixSpecifier, BaseItemId, ItemLevel, ItemRarityEnum, THashSet},
    utils::hash_utils::hash_set_unordered,
};

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct ItemTechnicalMeta {}

#[derive(Debug, Clone)]
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
