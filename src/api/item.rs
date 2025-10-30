use std::collections::HashSet;

use crate::api::types::{AffixSpecifier, BaseItemId, ItemLevel, ItemRarityEnum};

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[derive(Debug, PartialEq)]
pub struct ItemSnapshot {
    pub item_level: ItemLevel,
    pub rarity: ItemRarityEnum,
    pub base_id: BaseItemId,
    pub affixes: HashSet<AffixSpecifier>,
}
