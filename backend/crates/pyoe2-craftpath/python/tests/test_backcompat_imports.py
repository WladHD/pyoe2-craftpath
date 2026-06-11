"""Guards the backward-compatible import surface after the _native rename."""

import pyoe2_craftpath as pc

EXPECTED_NATIVE_NAMES = [
    # types
    "AffixId", "AffixDefinition", "AffixClassEnum", "AffixLocationEnum",
    "AffixSpecifier", "AffixTierConstraints", "AffixTierLevel", "AffixTierLevelMeta",
    "BaseItemId", "BaseGroupId", "BaseGroupDefinition", "ItemName", "ItemId",
    "Item", "ItemSnapshot", "ItemSnapshotHelper", "ItemTechnicalMeta",
    "ItemMatrixNode", "ItemRoute", "ItemRouteNode",
    # calculator / presets
    "Calculator", "DynMatrixBuilder", "MatrixBuilderPreset",
    "DynStatisticAnalyzerPaths", "DynStatisticAnalyzerCurrencyGroups",
    "StatisticAnalyzerPathPreset", "StatisticAnalyzerCurrencyGroupPreset",
    # currency / prices
    "CraftCurrencyEnum", "CraftCurrencyList", "PriceInDivines", "PriceKind",
    # providers
    "ItemInfoProvider", "MarketPriceProvider", "PoeNinjaMarketPriceProvider",
    "CraftOfExileItemInfoProvider", "CraftOfExileEmulatorItemImport",
    # essences / misc
    "EssenceId", "EssenceDefinition", "EssenceTierLevelMeta",
    "GroupRoute", "RouteChance", "RouteCustomWeight", "PropagationTarget",
    "StatisticResult", "Weight", "Fraction",
    "AffixTierLevelBoundsEnum", "ItemRarityEnum",
    # functions
    "retrieve_contents_from_urls_with_cache_unstable_order",
    "check_for_updates_and_print",
    # typed exceptions
    "CraftPathException", "TargetUnreachableError", "ItemUnreachableError",
    "RamLimitError", "ProviderDataError", "EssenceIntermediaryError",
]


def test_native_surface_complete():
    missing = [name for name in EXPECTED_NATIVE_NAMES if not hasattr(pc, name)]
    assert not missing, f"names lost from the public surface: {missing}"


def test_engine_layer_exposed():
    assert pc.LocalEngine is not None
    assert pc.JobSpec is not None
    assert pc.JobState.QUEUED.value == "JOB_STATE_QUEUED"
    # lazy attributes resolve
    from pyoe2_craftpath.engine import AsyncRemoteEngine, RemoteEngine  # noqa: F401


def test_exception_hierarchy():
    assert issubclass(pc.TargetUnreachableError, pc.CraftPathException)
    assert issubclass(pc.RamLimitError, pc.CraftPathException)
    assert issubclass(pc.CraftPathException, Exception)


def test_sanity_check_item_is_unexposed():
    # backing fn is todo!(); exposing it produced an opaque panic from Python
    assert not hasattr(pc.Calculator, "sanity_check_item")


def test_fraction_roundtrip():
    f = pc.Fraction(1, 4)
    assert f.num == 1 and f.den == 4
