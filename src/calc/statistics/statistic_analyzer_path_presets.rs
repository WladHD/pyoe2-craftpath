use serde::{Deserialize, Serialize};

use crate::{
    api::calculator::DynStatisticAnalyzerPaths,
    calc::statistics::analyzers::{
        all_path_chance_statistic_analyzer::AllUniquePathsChanceStatisticAnalyzer,
        unique_path_chance_statistic_analyzer::UniquePathChanceStatisticAnalyzer,
        unique_path_cost_statistic_analyzer::UniquePathCostStatisticAnalyzer,
        unique_path_efficient_cost_statistic_analyzer::UniquePathEfficientCostStatisticAnalyzer,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub enum StatisticAnalyzerPathPreset {
    UniquePathChance,
    UniquePathEfficiency,
    UniquePathCost,
    AllUniquePathChance,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl StatisticAnalyzerPathPreset {
    pub fn get_instance(&self) -> DynStatisticAnalyzerPaths {
        match self {
            // default
            &StatisticAnalyzerPathPreset::UniquePathChance => {
                DynStatisticAnalyzerPaths(Box::new(UniquePathChanceStatisticAnalyzer))
            }
            &StatisticAnalyzerPathPreset::UniquePathCost => {
                DynStatisticAnalyzerPaths(Box::new(UniquePathCostStatisticAnalyzer))
            }
            &StatisticAnalyzerPathPreset::UniquePathEfficiency => {
                DynStatisticAnalyzerPaths(Box::new(UniquePathEfficientCostStatisticAnalyzer))
            }
            // efficient for calc all
            &StatisticAnalyzerPathPreset::AllUniquePathChance => {
                DynStatisticAnalyzerPaths(Box::new(AllUniquePathsChanceStatisticAnalyzer))
            }
        }
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(StatisticAnalyzerPathPreset);
