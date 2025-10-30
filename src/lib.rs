pub mod api;
pub mod coe;
pub mod utils;

#[cfg(feature = "python")]
pub mod py {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3_stub_gen::define_stub_info_gatherer;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::api::provider::item_info::ItemInfoProvider;
    use crate::api::types::{AffixClassEnum, AffixDefinition, AffixId, AffixLocationEnum};
    use crate::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn parse_item_data_from_json(json: &str) -> PyResult<ItemInfoProvider> {
        CraftOfExileItemInfoProvider::parse_from_json(json)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[pymodule]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(parse_item_data_from_json, m)?)?;
        m.add_class::<AffixId>()?;
        m.add_class::<AffixDefinition>()?;
        m.add_class::<AffixClassEnum>()?;
        m.add_class::<AffixLocationEnum>()?;
        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
