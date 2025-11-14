use serde::{Deserialize, Serialize};

use crate::{
    api::calculator::DynStatisticAnalyzerCurrencyGroups,
    calc::statistics::analyzers::currency_groups::{
        currency_group_chance_statistic_analyzer::CurrencyGroupChanceStatisticAnalyzer,
        currency_group_efficient_chance_statistic_analyzer::CurrencyGroupChanceMemoryEfficientStatisticAnalyzer,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
/// Collection of presets provided from CraftPath by default
pub enum StatisticAnalyzerCurrencyGroupPreset {
    /// Collect ALL currency sequences that lead to the target item, sorted by chance
    /// Additionally, SUMS of all unique subpaths and returns ONE averaged out subpath
    CurrencyGroupChance,
    /// Collect ALL currency sequences that lead to the target item, sorted by chance
    /// Additionally, collects and returns ALL unique subpaths
    CurrencyGroupChanceMemoryHeavy,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl StatisticAnalyzerCurrencyGroupPreset {
    pub fn get_instance(&self) -> DynStatisticAnalyzerCurrencyGroups {
        match self {
            &StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance => {
                DynStatisticAnalyzerCurrencyGroups(Box::new(
                    CurrencyGroupChanceMemoryEfficientStatisticAnalyzer,
                ))
            }
            &StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy => {
                DynStatisticAnalyzerCurrencyGroups(Box::new(CurrencyGroupChanceStatisticAnalyzer))
            }
        }
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(StatisticAnalyzerCurrencyGroupPreset);
