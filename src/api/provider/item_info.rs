use crate::api::types::{
    AffixDefinition, AffixId, AffixTierLevel, AffixTierLevelMeta, BaseItemId, EssenceDefinition,
    EssenceId, THashMap,
};

pub type AffixWeightTable = THashMap<AffixId, THashMap<AffixTierLevel, AffixTierLevelMeta>>;

#[cfg(not(feature = "python"))]
pub struct ItemInfoProvider {
    cache_affix_def: THashMap<AffixId, AffixDefinition>,
    cache_item_affix_table: THashMap<BaseItemId, AffixWeightTable>,
    cache_affix_essence_table: THashMap<AffixId, EssenceId>,
    cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub struct ItemInfoProvider {
    #[pyo3(get)]
    cache_affix_def: THashMap<AffixId, AffixDefinition>,
    #[pyo3(get)]
    cache_item_affix_table: THashMap<BaseItemId, AffixWeightTable>,
    #[pyo3(get)]
    cache_affix_essence_table: THashMap<AffixId, EssenceId>,
    #[pyo3(get)]
    cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
}

// todo will include more info instead of raw ref to hm
impl ItemInfoProvider {
    pub fn new(
        cache_affix_def: THashMap<AffixId, AffixDefinition>,
        cache_item_affix_table: THashMap<BaseItemId, AffixWeightTable>,
        cache_affix_essence_table: THashMap<AffixId, EssenceId>,
        cache_essence_def: THashMap<EssenceId, EssenceDefinition>,
    ) -> Self {
        Self {
            cache_affix_def,
            cache_item_affix_table,
            cache_affix_essence_table,
            cache_essence_def,
        }
    }

    pub fn get_cache_affix_def(&self) -> &THashMap<AffixId, AffixDefinition> {
        &self.cache_affix_def
    }
    pub fn get_cache_item_affix_table(&self) -> &THashMap<BaseItemId, AffixWeightTable> {
        &self.cache_item_affix_table
    }
    pub fn get_cache_affix_essence_table(&self) -> &THashMap<AffixId, EssenceId> {
        &self.cache_affix_essence_table
    }
    pub fn get_cache_essence_def(&self) -> &THashMap<EssenceId, EssenceDefinition> {
        &self.cache_essence_def
    }
}
