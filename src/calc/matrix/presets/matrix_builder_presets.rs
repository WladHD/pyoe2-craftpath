use serde::{Deserialize, Serialize};

use crate::{
    api::calculator::DynMatrixBuilder,
    calc::matrix::happy_path_impl::happy_path_matrix_builder_impl::HappyPathMatrixBuilderImpl,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
/// Collection of presets provided from CraftPath by default
pub enum MatrixBuilderPreset {
    /// Builds a tree-like structure for item propagation,
    /// while trying to stay on the 'Happy Path'
    HappyPathMatrixBuilder,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl MatrixBuilderPreset {
    pub fn get_instance(&self) -> DynMatrixBuilder {
        match self {
            MatrixBuilderPreset::HappyPathMatrixBuilder => {
                DynMatrixBuilder(Box::new(HappyPathMatrixBuilderImpl))
            }
        }
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(MatrixBuilderPreset);
