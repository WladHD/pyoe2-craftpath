"""LocalEngine: the classic in-process flow behind the JobSpec interface."""

from __future__ import annotations

import threading
from typing import Any

from ._convert import _raw
from ._result import CraftResult, GroupResultView, GroupView, PathResultView, RouteView
from ._spec import JobSpec, JobState

_COE_URL = "https://www.craftofexile.com/json/poe2/main/poec_data.json"
_PN_URL = (
    "https://poe.ninja/poe2/api/economy/exchange/current/overview?league={league}&type={kind}"
)
_PN_KINDS = ("Abyss", "Currency", "Essences", "Ritual")


class LocalEngine:
    """Runs calculations in-process via the native Rust module.

    League data (CraftOfExile + poe.ninja) is fetched lazily per league and
    cached on disk in ``cache_dir`` (same behavior as the classic examples).
    """

    def __init__(
        self,
        *,
        cache_dir: str = "./cache",
        coe_cache_ttl: int = 24 * 60 * 60,
        economy_cache_ttl: int = 60 * 60,
    ) -> None:
        self.cache_dir = cache_dir
        self.coe_cache_ttl = coe_cache_ttl
        self.economy_cache_ttl = economy_cache_ttl
        self._providers: dict[str, tuple[Any, Any]] = {}

    # -- provider loading ---------------------------------------------------

    def providers(self, league: str) -> tuple[Any, Any]:
        """(ItemInfoProvider, MarketPriceProvider) for a league, cached."""
        if league not in self._providers:
            self._providers[league] = self._load_providers(league)
        return self._providers[league]

    def _load_providers(self, league: str) -> tuple[Any, Any]:
        import os

        from .._native import (
            CraftOfExileItemInfoProvider,
            PoeNinjaMarketPriceProvider,
            retrieve_contents_from_urls_with_cache_unstable_order,
        )

        league_dir = os.path.join(self.cache_dir, league.replace("/", "_").replace(" ", "_"))
        os.makedirs(league_dir, exist_ok=True)

        coe_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            {os.path.join(self.cache_dir, "coe2.json"): _COE_URL},
            self.coe_cache_ttl,
        )
        economy_jsons = retrieve_contents_from_urls_with_cache_unstable_order(
            {
                os.path.join(league_dir, f"pn_{kind.lower()}.json"): _PN_URL.format(
                    league=league, kind=kind
                )
                for kind in _PN_KINDS
            },
            self.economy_cache_ttl,
        )

        item_provider = CraftOfExileItemInfoProvider.parse_from_json(coe_jsons[0])
        market_provider = PoeNinjaMarketPriceProvider.parse_from_json_list(economy_jsons)
        return item_provider, market_provider

    # -- execution ----------------------------------------------------------

    def run(self, spec: JobSpec) -> CraftResult:
        """Run synchronously and return the unified result."""
        return _run_local(self, spec)

    def submit(self, spec: JobSpec) -> "LocalJob":
        """Run in a background thread (the native code detaches the GIL)."""
        return LocalJob(self, spec)


class LocalJob:
    """Job handle matching the RemoteJob surface where it can."""

    def __init__(self, engine: LocalEngine, spec: JobSpec) -> None:
        self._result: CraftResult | None = None
        self._error: BaseException | None = None
        self.state = JobState.RUNNING

        def target() -> None:
            try:
                self._result = _run_local(engine, spec)
                self.state = JobState.SUCCEEDED
            except BaseException as exc:  # noqa: BLE001 — stored, re-raised in wait()
                self._error = exc
                self.state = JobState.FAILED

        self._thread = threading.Thread(target=target, daemon=True)
        self._thread.start()

    def status(self) -> JobState:
        return self.state

    def wait(self, *, timeout: float | None = None) -> CraftResult:
        self._thread.join(timeout)
        if self._thread.is_alive():
            raise TimeoutError("local calculation still running")
        if self._error is not None:
            raise self._error
        assert self._result is not None
        return self._result

    def cancel(self) -> None:
        raise NotImplementedError(
            "LocalEngine jobs cannot be cancelled programmatically; "
            "interrupt with Ctrl-C (KeyboardInterrupt) or use RemoteEngine."
        )


def _steps_from_native_route(route: Any, item_provider: Any) -> list[list[str]]:
    steps = []
    for node in route.route:
        steps.append(sorted(c.get_item_name(item_provider) for c in node.currency_list.list))
    return steps


def _steps_from_native_group(group: Any, item_provider: Any) -> list[list[str]]:
    steps = []
    for currency_list in group.group:
        steps.append(sorted(c.get_item_name(item_provider) for c in currency_list.list))
    return steps


def _run_local(engine: LocalEngine, spec: JobSpec) -> CraftResult:
    from .._native import Calculator

    item_provider, market_provider = engine.providers(spec.league)

    calculator = Calculator.generate_item_matrix(
        spec.start,
        spec.target,
        item_provider,
        market_provider,
        spec.resolved_matrix_builder().get_instance(),
    )

    pretty_text_parts: list[str] = []
    top_n_pretty = spec.top_n_pretty if spec.top_n_pretty is not None else spec.max_routes

    # groups first so route rendering can reference them (mirrors CLI/worker)
    group_results: list[GroupResultView] = []
    native_groups_first: Any = None
    for preset in spec.group_analyzers:
        instance = preset.get_instance()
        groups = calculator.calculate_statistics_currency_group(
            item_provider, market_provider, spec.max_ram_in_bytes, instance
        )
        if native_groups_first is None:
            native_groups_first = groups

        views = []
        for i, group in enumerate(groups):
            pretty = None
            if spec.include_pretty_strings and i < top_n_pretty:
                pretty = group.to_pretty_string(item_provider, market_provider, instance)
                pretty_text_parts.append(pretty)
            views.append(
                GroupView(
                    chance=_raw(group.chance),
                    weight=_raw(group.weight),
                    amount_subpaths=_raw(group.amount_subpaths),
                    steps=_steps_from_native_group(group, item_provider),
                    pretty=pretty,
                    raw=group,
                )
            )
        group_results.append(
            GroupResultView(
                analyzer_name=instance.get_name(),
                unit_type=instance.get_unit_type(),
                lower_is_better=instance.lower_is_better(),
                groups=views,
            )
        )

    path_results: list[PathResultView] = []
    for preset in spec.path_analyzers:
        instance = preset.get_instance()
        routes = calculator.calculate_statistics(
            item_provider,
            market_provider,
            spec.max_routes,
            spec.max_ram_in_bytes,
            instance,
        )

        if spec.include_pretty_strings:
            pretty_text_parts.append(f"\n===== Results for '{instance.get_name()}' =====")

        views = []
        for i, route in enumerate(routes):
            pretty = None
            if spec.include_pretty_strings and i < top_n_pretty:
                pretty = route.to_pretty_string(
                    item_provider,
                    market_provider,
                    instance,
                    calculator,
                    native_groups_first,
                )
                pretty_text_parts.append(pretty)
            views.append(
                RouteView(
                    chance=_raw(route.chance),
                    weight=_raw(route.weight),
                    steps=_steps_from_native_route(route, item_provider),
                    pretty=pretty,
                    raw=route,
                )
            )
        path_results.append(
            PathResultView(
                analyzer_name=instance.get_name(),
                unit_type=instance.get_unit_type(),
                lower_is_better=instance.lower_is_better(),
                routes=views,
            )
        )

    return CraftResult(
        matrix_size=len(calculator.matrix),
        path_results=path_results,
        group_results=group_results,
        pretty_text="\n".join(pretty_text_parts),
        raw=calculator,
    )
