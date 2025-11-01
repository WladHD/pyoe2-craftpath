use anyhow::Result;

use crate::api::{
    errors::CraftPathError,
    types::{
        AffixDefinition, AffixId, AffixTierLevel, AffixTierLevelMeta, BaseItemId,
        EssenceDefinition, EssenceId, THashMap,
    },
};

pub type AffixWeightTable = THashMap<AffixId, THashMap<AffixTierLevel, AffixTierLevelMeta>>;

#[cfg(not(feature = "python"))]
pub struct ItemInfoProvider {
    cache_affix_def: THashMap<AffixId, AffixDefinition>,
    cache_item_affix_table: THashMap<BaseItemId, AffixWeightTable>,
    cache_affix_essence_table: THashMap<AffixId, EssenceId>,
    cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(weakref, from_py_object, frozen, get_all, str)
)]
pub struct ItemInfoProvider {
    pub cache_affix_def: THashMap<AffixId, AffixDefinition>,
    pub cache_item_affix_table: THashMap<BaseItemId, AffixWeightTable>,
    pub cache_affix_essence_table: THashMap<AffixId, EssenceId>,
    pub cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
}

impl ItemInfoProvider {
    pub fn lookup_base_item_mods(&self, base_item: &BaseItemId) -> Result<&AffixWeightTable> {
        self.cache_item_affix_table
            .get(&base_item)
            .ok_or_else(|| CraftPathError::ItemWithoutAffixInformation(base_item.clone()).into())
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl ItemInfoProvider {
    pub fn cloned_lookup_base_item_mods(
        &self,
        base_item: &BaseItemId,
    ) -> pyo3::PyResult<AffixWeightTable> {
        self.lookup_base_item_mods(&base_item)
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }
}

#[cfg(feature = "python")]
impl std::fmt::Display for ItemInfoProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ItemInfoProvider: (cache_affix_def: {}), cache_affix_essence_table {}, cache_essence_def {}, cache_item_affix_table {}",
            self.cache_affix_def.len(),
            self.cache_affix_essence_table.len(),
            self.cache_essence_def.len(),
            self.cache_item_affix_table.len()
        )
    }
}
