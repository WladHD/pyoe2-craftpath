from __future__ import annotations

import enum
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .._native import (
        ItemSnapshot,
        MatrixBuilderPreset,
        StatisticAnalyzerCurrencyGroupPreset,
        StatisticAnalyzerPathPreset,
    )


class JobState(str, enum.Enum):
    """Lifecycle states, wire-compatible with craftpath.v1.JobState."""

    UNSPECIFIED = "JOB_STATE_UNSPECIFIED"
    QUEUED = "JOB_STATE_QUEUED"
    RUNNING = "JOB_STATE_RUNNING"
    SUCCEEDED = "JOB_STATE_SUCCEEDED"
    FAILED = "JOB_STATE_FAILED"
    CANCELLED = "JOB_STATE_CANCELLED"
    EXPIRED = "JOB_STATE_EXPIRED"

    @property
    def is_terminal(self) -> bool:
        return self in (
            JobState.SUCCEEDED,
            JobState.FAILED,
            JobState.CANCELLED,
            JobState.EXPIRED,
        )

    @classmethod
    def parse(cls, value: str) -> "JobState":
        try:
            return cls(value)
        except ValueError:
            return cls.UNSPECIFIED


def _default_path_analyzers() -> list["StatisticAnalyzerPathPreset"]:
    from .._native import StatisticAnalyzerPathPreset

    return [StatisticAnalyzerPathPreset.UniquePathChance]


@dataclass
class JobSpec:
    """One calculation request, usable with both engines.

    ``start`` / ``target`` are native :class:`ItemSnapshot` objects (e.g. from
    ``CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string``).
    """

    start: "ItemSnapshot"
    target: "ItemSnapshot"
    league: str = "Standard"
    matrix_builder: "MatrixBuilderPreset | None" = None
    path_analyzers: list["StatisticAnalyzerPathPreset"] = field(
        default_factory=_default_path_analyzers
    )
    group_analyzers: list["StatisticAnalyzerCurrencyGroupPreset"] = field(default_factory=list)
    max_routes: int = 5
    max_ram_in_bytes: int = 1_000_000_000
    timeout_seconds: int | None = None
    include_pretty_strings: bool = True
    include_route_snapshots: bool = False
    top_n_pretty: int | None = None

    def resolved_matrix_builder(self) -> "MatrixBuilderPreset":
        if self.matrix_builder is not None:
            return self.matrix_builder
        from .._native import MatrixBuilderPreset

        return MatrixBuilderPreset.HappyPathMatrixBuilder
