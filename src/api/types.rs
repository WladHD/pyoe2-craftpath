use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::explicit_type;

pub type THashMap<K, V> = FxHashMap<K, V>;
pub type THashSet<K> = FxHashSet<K>;

// MODS
explicit_type!(Weight, u32);
explicit_type!(AffixTierLevel, u8);
explicit_type!(EssenceId, u16);
explicit_type!(AffixId, u16);
// ITEMS
explicit_type!(ItemId, u16);
explicit_type!(BaseGroupId, u16);
explicit_type!(BaseItemId, u16);
explicit_type!(ItemLevel, u8);

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub enum ItemRarityEnum {
    Normal = 0,
    Magic = 1,
    Rare = 2,
    Unique = 3,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub enum AffixTierLevelBoundsEnum {
    Exact,
    Minimum,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
#[repr(u8)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub enum AffixClassEnum {
    Base = 1,
    Desecrated = 10,
    Essence = 13,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub enum AffixLocationEnum {
    Socket,
    Prefix,
    Suffix,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub struct AffixTierConstraints {
    pub tier: AffixTierLevel,
    pub bounds: AffixTierLevelBoundsEnum,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub struct AffixSpecifier {
    pub affix: AffixId,
    pub fractured: bool,
    pub tier: AffixTierConstraints,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub struct AffixTierLevelMeta {
    pub weight: Weight,
    pub min_item_level: ItemLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, get_all, str))]
pub struct AffixDefinition {
    /// 2 Affixes with intersecting exlusive groups
    /// cannot be applied on same item
    pub exlusive_groups: THashSet<String>,
    /// Tags like "Physical" etc. for Homogen Omen
    pub tags: THashSet<u8>,
    pub description_template: String,
    /// Normal, Desecrated, Essence
    pub affix_class: AffixClassEnum,
    /// Prefix, Suffix, Socket
    pub affix_location: AffixLocationEnum,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct EssenceDefinition {
    pub name_essence: String,
    pub base_tier_table: THashMap<BaseItemId, THashMap<AffixId, EssenceTierLevelMeta>>,
    pub corrupt: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct BaseGroupDefinition {
    pub name_base_group: String,
    pub max_affix: u8,
    pub max_sockets: u8,
    pub is_rare: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct EssenceTierLevelMeta {
    // mod name
    // pub affix_id: AffixId,
    // modgroup?
    pub id: String, // todo to newtype?
    pub min_item_level: ItemLevel,
}

// generate debug display output for python automatically.
// can specialcase items later on
#[cfg(feature = "python")]
crate::derive_DebugDisplay!(
    AffixDefinition,
    EssenceDefinition,
    EssenceTierLevelMeta,
    ItemRarityEnum,
    AffixTierLevelBoundsEnum,
    AffixClassEnum,
    AffixLocationEnum,
    AffixTierConstraints,
    AffixSpecifier,
    AffixTierLevelMeta,
    BaseGroupDefinition
);
