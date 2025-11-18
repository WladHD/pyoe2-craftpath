pub mod api;
pub mod calc;
pub mod external_api;
pub mod utils;

pub const GITHUB_REPOSITORY: &str = "WladHD/pyoe2-craftpath";

#[cfg(feature = "python")]
pub mod py {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3_stub_gen::define_stub_info_gatherer;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::GITHUB_REPOSITORY;
    use crate::api::calculator::{
        Calculator, DynMatrixBuilder, DynStatisticAnalyzerCurrencyGroups,
        DynStatisticAnalyzerPaths, GroupRoute, ItemMatrixNode, ItemRoute, ItemRouteNode,
        PropagationTarget, StatisticResult,
    };
    use crate::api::currency::{CraftCurrencyEnum, CraftCurrencyList};
    use crate::api::item::{Item, ItemSnapshot, ItemSnapshotHelper, ItemTechnicalMeta};
    use crate::api::provider::item_info::ItemInfoProvider;
    use crate::api::provider::market_prices::{
        ItemName, MarketPriceProvider, PriceInDivines, PriceKind,
    };
    use crate::api::types::{
        AffixClassEnum, AffixDefinition, AffixId, AffixLocationEnum, AffixSpecifier,
        AffixTierConstraints, AffixTierLevel, AffixTierLevelBoundsEnum, AffixTierLevelMeta,
        BaseGroupDefinition, BaseGroupId, BaseItemId, EssenceDefinition, EssenceId,
        EssenceTierLevelMeta, ItemId, ItemRarityEnum, THashMap, Weight,
    };
    use crate::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
    use crate::calc::statistics::helpers::{RouteChance, RouteCustomWeight};
    use crate::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
    use crate::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
    use crate::external_api::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
    use crate::external_api::coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport;
    use crate::external_api::pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider;
    use crate::utils::fraction_utils::Fraction;
    use crate::utils::logger_utils::init_tracing;
    use crate::utils::version_checker_utils::check_new_version;

    #[gen_stub_pyfunction]
    #[pyfunction]
    /**
     * Order-preservation of `cache_url_map` is not guaranteed.
     * If order is required, split requests into single function calls.
     * E. g. Group 1. item info, Group 2. economy.
     */
    fn retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map: THashMap<String, String>,
        max_cache_duration_in_sec: u64,
    ) -> PyResult<Vec<String>> {
        crate::external_api::fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order(
            cache_url_map,
            max_cache_duration_in_sec,
        )
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn check_for_updates_and_print() -> PyResult<bool> {
        check_new_version(GITHUB_REPOSITORY).map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    #[pymodule]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        init_tracing();

        ctrlc::set_handler(|| std::process::exit(2)).unwrap();

        // Affix classes
        m.add_class::<AffixId>()?;
        m.add_class::<AffixDefinition>()?;
        m.add_class::<AffixClassEnum>()?;
        m.add_class::<AffixLocationEnum>()?;
        m.add_class::<AffixSpecifier>()?;
        m.add_class::<AffixTierConstraints>()?;
        m.add_class::<AffixTierLevel>()?;
        m.add_class::<AffixTierLevelMeta>()?;

        // Item classes
        m.add_class::<BaseItemId>()?;
        m.add_class::<BaseGroupId>()?;
        m.add_class::<BaseGroupDefinition>()?;
        m.add_class::<ItemName>()?;
        m.add_class::<ItemId>()?;
        m.add_class::<Item>()?;
        m.add_class::<ItemSnapshot>()?;
        m.add_class::<ItemSnapshotHelper>()?;
        m.add_class::<ItemTechnicalMeta>()?;
        m.add_class::<ItemMatrixNode>()?;
        m.add_class::<ItemRoute>()?;
        m.add_class::<ItemRouteNode>()?;

        // Calculator / matrix
        m.add_class::<Calculator>()?;
        m.add_class::<DynMatrixBuilder>()?;
        m.add_class::<MatrixBuilderPreset>()?;

        // Statistics analyzers
        m.add_class::<DynStatisticAnalyzerPaths>()?;
        m.add_class::<DynStatisticAnalyzerCurrencyGroups>()?;
        m.add_class::<StatisticAnalyzerPathPreset>()?;
        m.add_class::<StatisticAnalyzerCurrencyGroupPreset>()?;

        // Currency / prices
        m.add_class::<CraftCurrencyEnum>()?;
        m.add_class::<CraftCurrencyList>()?;
        m.add_class::<PriceInDivines>()?;
        m.add_class::<PriceKind>()?;

        // Providers
        m.add_class::<ItemInfoProvider>()?;
        m.add_class::<MarketPriceProvider>()?;
        m.add_class::<PoeNinjaMarketPriceProvider>()?;
        m.add_class::<CraftOfExileItemInfoProvider>()?;
        m.add_class::<CraftOfExileEmulatorItemImport>()?;

        // Essence classes
        m.add_class::<EssenceId>()?;
        m.add_class::<EssenceDefinition>()?;
        m.add_class::<EssenceTierLevelMeta>()?;

        // Misc / route
        m.add_class::<GroupRoute>()?;
        m.add_class::<RouteChance>()?;
        m.add_class::<RouteCustomWeight>()?;
        m.add_class::<PropagationTarget>()?;
        m.add_class::<StatisticResult>()?;
        m.add_class::<Weight>()?;
        m.add_class::<Fraction>()?;

        // Enums
        m.add_class::<AffixTierLevelBoundsEnum>()?;
        m.add_class::<ItemRarityEnum>()?;

        // general utility
        m.add_function(wrap_pyfunction!(
            retrieve_contents_from_urls_with_cache_unstable_order,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(check_for_updates_and_print, m)?)?;

        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}
