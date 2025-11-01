use crate::api::calculator::DynStatisticAnalyzer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub enum StatisticAnalyzerPreset {
    UniquePathChance,
    UniquePathEfficiency,
    UniquePathCost,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl StatisticAnalyzerPreset {
    pub fn get_statistic_analyzer_instance(&self) -> DynStatisticAnalyzer {
        todo!();
        // match self {
        //     StatisticCalculatorPreset::HappyPathMatrixBuilder => {
        //         DynMatrixBuilder(Box::new(HappyPathMatrixBuilderImpl))
        //     }
        // }
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(StatisticAnalyzerPreset);
