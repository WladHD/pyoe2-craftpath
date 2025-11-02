use anyhow::Result;

use crate::api::types::THashSet;
use crate::api::{
    errors::CraftPathError,
    types::{
        AffixDefinition, AffixId, AffixTierLevel, AffixTierLevelMeta, BaseItemId,
        EssenceDefinition, EssenceId, THashMap,
    },
};

pub type AffixWeightTable = THashMap<AffixId, THashMap<AffixTierLevel, AffixTierLevelMeta>>;

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
    pub cache_affix_essence_table: THashMap<AffixId, THashSet<EssenceId>>,
    pub cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
}

impl ItemInfoProvider {
    pub fn lookup_base_item_mods(&self, base_item: &BaseItemId) -> Result<&AffixWeightTable> {
        self.cache_item_affix_table
            .get(&base_item)
            .ok_or_else(|| CraftPathError::ItemWithoutAffixInformation(base_item.clone()).into())
    }

    pub fn lookup_affix_definition(&self, affix_id: &AffixId) -> Result<&AffixDefinition> {
        self.cache_affix_def
            .get(&affix_id)
            .ok_or_else(|| CraftPathError::AffixWithoutDefinition(affix_id.clone()).into())
    }

    pub fn lookup_essence_definition(&self, essence_id: &EssenceId) -> Result<&EssenceDefinition> {
        self.cache_essence_def
            .get(&essence_id)
            .ok_or_else(|| CraftPathError::EssenceWithoutDefinition(essence_id.clone()).into())
    }

    pub fn lookup_affix_essence(&self, affix_id: &AffixId) -> Result<&THashSet<EssenceId>> {
        self.cache_affix_essence_table
            .get(&affix_id)
            .ok_or_else(|| CraftPathError::AffixWithoutEssence(affix_id.clone()).into())
    }

    pub fn is_abyssal_mark(&self, id: &AffixId) -> bool {
        // parse dynamically?
        id == &AffixId::from(6160) || id == &AffixId::from(6159)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl ItemInfoProvider {
    #[pyo3(name = "lookup_base_item_mods")]
    pub fn lookup_base_item_mods_py(
        &self,
        base_item: &BaseItemId,
    ) -> pyo3::PyResult<AffixWeightTable> {
        self.lookup_base_item_mods(&base_item)
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[pyo3(name = "lookup_affix_definition")]
    pub fn lookup_affix_definition_py(
        &self,
        affix_id: &AffixId,
    ) -> pyo3::PyResult<AffixDefinition> {
        self.lookup_affix_definition(affix_id)
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[pyo3(name = "lookup_essence_definition")]
    pub fn lookup_essence_definition_py(
        &self,
        essence_id: &EssenceId,
    ) -> pyo3::PyResult<EssenceDefinition> {
        self.lookup_essence_definition(essence_id)
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[pyo3(name = "lookup_affix_essence")]
    pub fn lookup_affix_essence_py(
        &self,
        affix_id: &AffixId,
    ) -> pyo3::PyResult<THashSet<EssenceId>> {
        self.lookup_affix_essence(affix_id)
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[pyo3(name = "is_abyssal_mark")]
    pub fn is_abyssal_mark_py(&self, id: &AffixId) -> bool {
        // parse dynamically?
        id == &AffixId::from(6160) || id == &AffixId::from(6159)
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
