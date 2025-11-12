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
    use crate::api::provider::market_prices::{ItemName, PriceInDivines, PriceKind};
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
    /**
     * Order-preservation `cache_url_map` is not guaranteed.
     * If order is required, split requests into single groups.
     */
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

        // providers
        m.add_class::<PoeNinjaMarketPriceProvider>()?;
        m.add_class::<CraftOfExileEmulatorItemImport>()?;
        m.add_class::<CraftOfExileItemInfoProvider>()?;

        // general utility
        m.add_function(wrap_pyfunction!(retrieve_jsons_from_urls_with_cache, m)?)?;

        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
