pub mod api;
pub mod external_api;
pub mod utils;

#[cfg(feature = "python")]
pub mod py {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3_stub_gen::define_stub_info_gatherer;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::api::currency::CraftCurrencyEnum;
    use crate::api::provider::item_info::ItemInfoProvider;
    use crate::api::provider::market_prices::{
        ItemName, MarketPriceProvider, PriceInDivines, PriceKind,
    };
    use crate::api::types::{
        AffixClassEnum, AffixDefinition, AffixId, AffixLocationEnum, THashMap,
    };
    use crate::external_api::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
    use crate::external_api::pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider;
    use crate::utils::logger_utils::init_tracing;

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn parse_item_data_from_json(json: &str) -> PyResult<ItemInfoProvider> {
        CraftOfExileItemInfoProvider::parse_from_json(json)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn parse_economy_from_jsons(json: Vec<String>) -> PyResult<MarketPriceProvider> {
        PoeNinjaMarketPriceProvider::parse_from_json(json.as_slice())
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn retrieve_jsons_from_urls_with_cache(
        cache_url_map: THashMap<String, String>,
        max_cache_duration_in_sec: u64,
    ) -> PyResult<Vec<String>> {
        crate::external_api::fetch_json_from_urls::retrieve_jsons_from_urls_with_cache(
            cache_url_map,
            max_cache_duration_in_sec,
        )
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[pymodule]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        init_tracing();

        m.add_function(wrap_pyfunction!(parse_item_data_from_json, m)?)?;
        m.add_class::<AffixId>()?;
        m.add_class::<AffixDefinition>()?;
        m.add_class::<AffixClassEnum>()?;
        m.add_class::<AffixLocationEnum>()?;

        m.add_function(wrap_pyfunction!(parse_economy_from_jsons, m)?)?;
        m.add_class::<ItemName>()?;

        m.add_class::<PriceInDivines>()?;
        m.add_class::<PriceKind>()?;

        m.add_class::<CraftCurrencyEnum>()?;

        // general utility
        m.add_function(wrap_pyfunction!(retrieve_jsons_from_urls_with_cache, m)?)?;

        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
