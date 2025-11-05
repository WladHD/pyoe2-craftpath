pub mod api;
pub mod calc;
pub mod external_api;
pub mod utils;

#[cfg(feature = "python")]
pub mod py {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3_stub_gen::define_stub_info_gatherer;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::api::calculator::{
        Calculator, DynMatrixBuilder, DynStatisticAnalyzerCurrencyGroups, DynStatisticAnalyzerPaths,
    };
    use crate::api::currency::CraftCurrencyEnum;
    use crate::api::item::ItemSnapshot;
    use crate::api::provider::item_info::ItemInfoProvider;
    use crate::api::provider::market_prices::{
        ItemName, MarketPriceProvider, PriceInDivines, PriceKind,
    };
    use crate::api::types::{
        AffixClassEnum, AffixDefinition, AffixId, AffixLocationEnum, BaseItemId, EssenceDefinition,
        EssenceId, THashMap,
    };
    use crate::calc::matrix::matrix_builder_presets::MatrixBuilderPreset;
    use crate::calc::statistics::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
    use crate::calc::statistics::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
    use crate::external_api::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
    use crate::external_api::coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport;
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

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn parse_itemsnapshot_from_string(
        item_emulator_json: String,
        provider: &ItemInfoProvider,
    ) -> PyResult<ItemSnapshot> {
        CraftOfExileEmulatorItemImport::parse_itemsnapshot_from_string(
            item_emulator_json.as_str(),
            provider,
        )
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[pymodule]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        init_tracing();

        ctrlc::set_handler(|| std::process::exit(2)).unwrap();

        m.add_class::<AffixId>()?;
        m.add_class::<AffixDefinition>()?;
        m.add_class::<AffixClassEnum>()?;
        m.add_class::<AffixLocationEnum>()?;

        m.add_class::<ItemName>()?;

        m.add_class::<PriceInDivines>()?;
        m.add_class::<PriceKind>()?;

        m.add_class::<CraftCurrencyEnum>()?;

        m.add_class::<Calculator>()?;
        m.add_class::<MatrixBuilderPreset>()?;
        m.add_class::<DynMatrixBuilder>()?;
        m.add_class::<DynStatisticAnalyzerPaths>()?;
        m.add_class::<DynStatisticAnalyzerCurrencyGroups>()?;
        m.add_class::<StatisticAnalyzerPathPreset>()?;
        m.add_class::<StatisticAnalyzerCurrencyGroupPreset>()?;

        m.add_class::<BaseItemId>()?;
        m.add_class::<EssenceId>()?;
        m.add_class::<EssenceDefinition>()?;

        // general utility
        m.add_function(wrap_pyfunction!(retrieve_jsons_from_urls_with_cache, m)?)?;
        m.add_function(wrap_pyfunction!(parse_item_data_from_json, m)?)?;
        m.add_function(wrap_pyfunction!(parse_economy_from_jsons, m)?)?;
        m.add_function(wrap_pyfunction!(parse_itemsnapshot_from_string, m)?)?;

        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
