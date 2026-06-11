#[cfg(feature = "python")]
pub mod py {
    use pyo3::prelude::*;
    use pyo3_stub_gen::define_stub_info_gatherer;
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use craftpath_core::GITHUB_REPOSITORY;
    use craftpath_core::api::calculator::{
        Calculator, DynMatrixBuilder, DynStatisticAnalyzerCurrencyGroups,
        DynStatisticAnalyzerPaths, GroupRoute, ItemMatrixNode, ItemRoute, ItemRouteNode,
        PropagationTarget, StatisticResult,
    };
    use craftpath_core::api::currency::{CraftCurrencyEnum, CraftCurrencyList};
    use craftpath_core::api::item::{Item, ItemSnapshot, ItemSnapshotHelper, ItemTechnicalMeta};
    use craftpath_core::api::provider::item_info::ItemInfoProvider;
    use craftpath_core::api::provider::market_prices::{
        ItemName, MarketPriceProvider, PriceInDivines, PriceKind,
    };
    use craftpath_core::api::types::{
        AffixClassEnum, AffixDefinition, AffixId, AffixLocationEnum, AffixSpecifier,
        AffixTierConstraints, AffixTierLevel, AffixTierLevelBoundsEnum, AffixTierLevelMeta,
        BaseGroupDefinition, BaseGroupId, BaseItemId, EssenceDefinition, EssenceId,
        EssenceTierLevelMeta, ItemId, ItemRarityEnum, THashMap, Weight,
    };
    use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
    use craftpath_core::calc::statistics::helpers::{RouteChance, RouteCustomWeight};
    use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
    use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
    use craftpath_core::external_api::coe::craftofexile_data_provider_adapter::CraftOfExileItemInfoProvider;
    use craftpath_core::external_api::coe_emulator::coe_emulator_item_snapshot_provider::CraftOfExileEmulatorItemImport;
    use craftpath_core::external_api::pn::poe_ninja_data_provider_adapter::PoeNinjaMarketPriceProvider;
    use craftpath_core::utils::fraction_utils::Fraction;
    use craftpath_core::utils::logger_utils::init_tracing;
    use craftpath_core::utils::py_err_utils::{
        CraftPathException, EssenceIntermediaryError, ItemUnreachableError, ProviderDataError,
        RamLimitError, TargetUnreachableError, to_py_err,
    };
    use craftpath_core::utils::version_checker_utils::check_new_version;

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
        craftpath_core::external_api::fetch_json_from_urls::retrieve_contents_from_urls_with_cache_unstable_order(
            cache_url_map,
            max_cache_duration_in_sec,
        )
        .map_err(to_py_err)
    }

    #[gen_stub_pyfunction]
    #[pyfunction]
    fn check_for_updates_and_print() -> PyResult<bool> {
        check_new_version(GITHUB_REPOSITORY).map_err(to_py_err)
    }

    #[pymodule(name = "_native")]
    fn pyoe2_craftpath(m: &Bound<'_, PyModule>) -> PyResult<()> {
        init_tracing();

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

        // Exceptions
        m.add(
            "CraftPathException",
            m.py().get_type::<CraftPathException>(),
        )?;
        m.add(
            "TargetUnreachableError",
            m.py().get_type::<TargetUnreachableError>(),
        )?;
        m.add(
            "ItemUnreachableError",
            m.py().get_type::<ItemUnreachableError>(),
        )?;
        m.add("RamLimitError", m.py().get_type::<RamLimitError>())?;
        m.add("ProviderDataError", m.py().get_type::<ProviderDataError>())?;
        m.add(
            "EssenceIntermediaryError",
            m.py().get_type::<EssenceIntermediaryError>(),
        )?;

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
