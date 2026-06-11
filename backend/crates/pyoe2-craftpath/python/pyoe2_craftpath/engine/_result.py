"""Engine-agnostic result views.

Both engines return :class:`CraftResult`; native objects (LocalEngine) or the
canonical-JSON payload (RemoteEngine) stay reachable via ``.raw``.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class RouteView:
    """One crafting route: an ordered list of currency applications."""

    chance: float
    weight: float
    steps: list[list[str]]
    """Per step: the display names of the currencies applied together."""
    pretty: str | None = None
    raw: Any = None

    def __str__(self) -> str:
        if self.pretty:
            return self.pretty
        lines = [f"Route (chance {self.chance:.5%}, weight {self.weight:g}):"]
        lines += [f"  {i + 1}. {' + '.join(step)}" for i, step in enumerate(self.steps)]
        return "\n".join(lines)


@dataclass
class GroupView:
    """One currency-sequence group (order of application, ignoring items)."""

    chance: float
    weight: float
    amount_subpaths: int
    steps: list[list[str]]
    pretty: str | None = None
    raw: Any = None

    def __str__(self) -> str:
        if self.pretty:
            return self.pretty
        lines = [
            f"Group (chance {self.chance:.5%}, {self.amount_subpaths} subpaths):",
        ]
        lines += [f"  {i + 1}. {' + '.join(step)}" for i, step in enumerate(self.steps)]
        return "\n".join(lines)


@dataclass
class PathResultView:
    analyzer_name: str
    unit_type: str
    lower_is_better: bool
    routes: list[RouteView] = field(default_factory=list)


@dataclass
class GroupResultView:
    analyzer_name: str
    unit_type: str
    lower_is_better: bool
    groups: list[GroupView] = field(default_factory=list)


@dataclass
class CraftResult:
    """Unified result of a calculation job."""

    matrix_size: int
    path_results: list[PathResultView] = field(default_factory=list)
    group_results: list[GroupResultView] = field(default_factory=list)
    pretty_text: str = ""
    raw: Any = None
    """LocalEngine: the native Calculator; RemoteEngine: the JobResult dict."""
