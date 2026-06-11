"""Conversions between native objects and the canonical proto3-JSON wire
shape (dicts). The dict form is what travels as ``application/json``; for
binary protobuf the client parses these dicts into the generated messages, so
there is exactly one conversion code path."""

from __future__ import annotations

from typing import Any


def _raw(value: Any):
    """Unwrap a native newtype regardless of accessor style (`raw_value`
    property, `raw_value()` method or `get_raw_value()` method)."""
    attr = getattr(value, "raw_value", None)
    if attr is not None:
        return attr() if callable(attr) else attr
    return value.get_raw_value()


def _rarity_to_proto(rarity: Any) -> str:
    from .._native import ItemRarityEnum

    mapping = {
        ItemRarityEnum.Normal: "ITEM_RARITY_NORMAL",
        ItemRarityEnum.Magic: "ITEM_RARITY_MAGIC",
        ItemRarityEnum.Rare: "ITEM_RARITY_RARE",
        ItemRarityEnum.Unique: "ITEM_RARITY_UNIQUE",
    }
    return mapping[rarity]


def _bounds_to_proto(bounds: Any) -> str:
    from .._native import AffixTierLevelBoundsEnum

    mapping = {
        AffixTierLevelBoundsEnum.Exact: "AFFIX_TIER_LEVEL_BOUNDS_EXACT",
        AffixTierLevelBoundsEnum.Minimum: "AFFIX_TIER_LEVEL_BOUNDS_MINIMUM",
    }
    return mapping[bounds]


def _affix_specifier_to_dict(spec: Any) -> dict:
    return {
        "affixId": int(_raw(spec.affix)),
        "fractured": bool(spec.fractured),
        "tier": {
            "tier": int(_raw(spec.tier.tier)),
            "bounds": _bounds_to_proto(spec.tier.bounds),
        },
    }


def snapshot_to_dict(snapshot: Any) -> dict:
    """Native ItemSnapshot -> canonical-JSON dict of craftpath.v1.ItemSnapshot."""
    return {
        "itemLevel": int(_raw(snapshot.item_level)),
        "rarity": _rarity_to_proto(snapshot.rarity),
        "baseId": int(_raw(snapshot.base_id)),
        "affixes": [_affix_specifier_to_dict(a) for a in snapshot.affixes],
        "corrupted": bool(snapshot.corrupted),
        "allowedSockets": int(snapshot.allowed_sockets),
        "sockets": [_affix_specifier_to_dict(a) for a in snapshot.sockets],
    }


def _enum_lookup(preset: Any, pairs: list[tuple[Any, str]], kind: str) -> str:
    # the native preset enums implement __eq__ but not __hash__, so a dict
    # lookup is not possible
    for native_value, proto_name in pairs:
        if preset == native_value:
            return proto_name
    raise ValueError(f"unknown {kind} preset: {preset!r}")


def _matrix_builder_to_proto(preset: Any) -> str:
    from .._native import MatrixBuilderPreset

    return _enum_lookup(
        preset,
        [(MatrixBuilderPreset.HappyPathMatrixBuilder, "MATRIX_BUILDER_PRESET_HAPPY_PATH")],
        "matrix builder",
    )


def _path_preset_to_proto(preset: Any) -> str:
    from .._native import StatisticAnalyzerPathPreset as P

    return _enum_lookup(
        preset,
        [
            (P.UniquePathChance, "STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE"),
            (P.UniquePathEfficiency, "STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_EFFICIENCY"),
            (P.UniquePathCost, "STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_COST"),
            (
                P.UniquePathChanceMemoryHeavy,
                "STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE_MEMORY_HEAVY",
            ),
        ],
        "path analyzer",
    )


def _group_preset_to_proto(preset: Any) -> str:
    from .._native import StatisticAnalyzerCurrencyGroupPreset as P

    return _enum_lookup(
        preset,
        [
            (
                P.CurrencyGroupChance,
                "STATISTIC_ANALYZER_CURRENCY_GROUP_PRESET_CURRENCY_GROUP_CHANCE",
            ),
            (
                P.CurrencyGroupChanceMemoryHeavy,
                "STATISTIC_ANALYZER_CURRENCY_GROUP_PRESET_CURRENCY_GROUP_CHANCE_MEMORY_HEAVY",
            ),
        ],
        "group analyzer",
    )


def job_spec_to_dict(spec: Any) -> dict:
    """JobSpec -> canonical-JSON dict of craftpath.v1.SubmitJobRequest."""
    body: dict = {
        "league": spec.league,
        "start": snapshot_to_dict(spec.start),
        "target": snapshot_to_dict(spec.target),
        "matrixBuilder": _matrix_builder_to_proto(spec.resolved_matrix_builder()),
        "pathAnalyzers": [_path_preset_to_proto(p) for p in spec.path_analyzers],
        "groupAnalyzers": [_group_preset_to_proto(p) for p in spec.group_analyzers],
        "limits": {
            "maxRoutes": spec.max_routes,
            # 64-bit ints are strings in canonical proto JSON
            "maxRamInBytes": str(spec.max_ram_in_bytes),
        },
        "resultOptions": {
            "includePrettyStrings": spec.include_pretty_strings,
            "includeRouteSnapshots": spec.include_route_snapshots,
        },
    }
    if spec.timeout_seconds is not None:
        body["limits"]["timeoutSeconds"] = spec.timeout_seconds
    if spec.top_n_pretty is not None:
        body["resultOptions"]["topNPretty"] = spec.top_n_pretty
    return body


# ---------------------------------------------------------------------------
# Wire result (canonical-JSON dict) -> result views
# ---------------------------------------------------------------------------


def _currency_list_steps(currency_lists: list[dict]) -> list[list[str]]:
    steps = []
    for cl in currency_lists:
        names = []
        for currency in cl.get("list", []):
            names.append(currency.get("displayName") or currency.get("kind", "?"))
        # CraftCurrencyList has set semantics; sort for stable output
        steps.append(sorted(names))
    return steps


def result_dict_to_views(result: dict) -> "CraftResult":
    from ._result import (
        CraftResult,
        GroupResultView,
        GroupView,
        PathResultView,
        RouteView,
    )

    path_results = []
    for pr in result.get("pathResults", []):
        routes = []
        for route in pr.get("routes", []):
            routes.append(
                RouteView(
                    chance=float(route.get("chance", 0.0)),
                    weight=float(route.get("weight", 0.0)),
                    steps=_currency_list_steps(
                        [n.get("currencyList", {}) for n in route.get("route", [])]
                    ),
                    pretty=route.get("pretty"),
                    raw=route,
                )
            )
        path_results.append(
            PathResultView(
                analyzer_name=pr.get("preset", ""),
                unit_type=pr.get("unitType", ""),
                lower_is_better=bool(pr.get("lowerIsBetter", False)),
                routes=routes,
            )
        )

    group_results = []
    for gr in result.get("groupResults", []):
        groups = []
        for group in gr.get("groups", []):
            groups.append(
                GroupView(
                    chance=float(group.get("chance", 0.0)),
                    weight=float(group.get("weight", 0.0)),
                    amount_subpaths=int(group.get("amountSubpaths", 0)),
                    steps=_currency_list_steps(group.get("group", [])),
                    pretty=group.get("pretty"),
                    raw=group,
                )
            )
        group_results.append(
            GroupResultView(
                analyzer_name=gr.get("preset", ""),
                unit_type=gr.get("unitType", ""),
                lower_is_better=bool(gr.get("lowerIsBetter", False)),
                groups=groups,
            )
        )

    return CraftResult(
        matrix_size=int(result.get("matrixSize", 0)),
        path_results=path_results,
        group_results=group_results,
        pretty_text=result.get("prettyText", ""),
        raw=result,
    )
