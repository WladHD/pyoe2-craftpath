use serde::{Deserialize, Serialize};

use crate::{
    api::calculator::DynStatisticAnalyzerCurrencyGroups,
    calc::statistics::analyzers::currency_group_chance_statistic_analyzer::CurrencyGroupChanceStatisticAnalyzer,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub enum StatisticAnalyzerCurrencyGroupPreset {
    CurrencyGroupChance,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl StatisticAnalyzerCurrencyGroupPreset {
    pub fn get_statistic_analyzer_instance(&self) -> DynStatisticAnalyzerCurrencyGroups {
        match self {
            &StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance => {
                DynStatisticAnalyzerCurrencyGroups(Box::new(CurrencyGroupChanceStatisticAnalyzer))
            } // _ => todo!(),
        }
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(StatisticAnalyzerCurrencyGroupPreset);
