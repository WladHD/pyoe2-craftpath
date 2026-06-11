from . import common_pb2 as _common_pb2
from . import currency_pb2 as _currency_pb2
from . import item_pb2 as _item_pb2
from . import presets_pb2 as _presets_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor
JOB_STATE_CANCELLED: JobState
JOB_STATE_EXPIRED: JobState
JOB_STATE_FAILED: JobState
JOB_STATE_QUEUED: JobState
JOB_STATE_RUNNING: JobState
JOB_STATE_SUCCEEDED: JobState
JOB_STATE_UNSPECIFIED: JobState

class GroupAnalyzerResult(_message.Message):
    __slots__ = ["groups", "lower_is_better", "preset", "unit_type"]
    GROUPS_FIELD_NUMBER: _ClassVar[int]
    LOWER_IS_BETTER_FIELD_NUMBER: _ClassVar[int]
    PRESET_FIELD_NUMBER: _ClassVar[int]
    UNIT_TYPE_FIELD_NUMBER: _ClassVar[int]
    groups: _containers.RepeatedCompositeFieldContainer[GroupRoute]
    lower_is_better: bool
    preset: _presets_pb2.StatisticAnalyzerCurrencyGroupPreset
    unit_type: str
    def __init__(self, preset: _Optional[_Union[_presets_pb2.StatisticAnalyzerCurrencyGroupPreset, str]] = ..., groups: _Optional[_Iterable[_Union[GroupRoute, _Mapping]]] = ..., lower_is_better: bool = ..., unit_type: _Optional[str] = ...) -> None: ...

class GroupRoute(_message.Message):
    __slots__ = ["amount_subpaths", "chance", "group", "pretty", "unique_route_weights", "weight"]
    AMOUNT_SUBPATHS_FIELD_NUMBER: _ClassVar[int]
    CHANCE_FIELD_NUMBER: _ClassVar[int]
    GROUP_FIELD_NUMBER: _ClassVar[int]
    PRETTY_FIELD_NUMBER: _ClassVar[int]
    UNIQUE_ROUTE_WEIGHTS_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    amount_subpaths: int
    chance: float
    group: _containers.RepeatedCompositeFieldContainer[_currency_pb2.CraftCurrencyList]
    pretty: str
    unique_route_weights: _containers.RepeatedCompositeFieldContainer[RouteChances]
    weight: float
    def __init__(self, group: _Optional[_Iterable[_Union[_currency_pb2.CraftCurrencyList, _Mapping]]] = ..., weight: _Optional[float] = ..., unique_route_weights: _Optional[_Iterable[_Union[RouteChances, _Mapping]]] = ..., chance: _Optional[float] = ..., amount_subpaths: _Optional[int] = ..., pretty: _Optional[str] = ...) -> None: ...

class ItemRoute(_message.Message):
    __slots__ = ["chance", "pretty", "route", "weight"]
    CHANCE_FIELD_NUMBER: _ClassVar[int]
    PRETTY_FIELD_NUMBER: _ClassVar[int]
    ROUTE_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    chance: float
    pretty: str
    route: _containers.RepeatedCompositeFieldContainer[ItemRouteNode]
    weight: float
    def __init__(self, route: _Optional[_Iterable[_Union[ItemRouteNode, _Mapping]]] = ..., weight: _Optional[float] = ..., chance: _Optional[float] = ..., pretty: _Optional[str] = ...) -> None: ...

class ItemRouteNode(_message.Message):
    __slots__ = ["chance", "currency_list", "item_matrix_id", "resolved_item"]
    CHANCE_FIELD_NUMBER: _ClassVar[int]
    CURRENCY_LIST_FIELD_NUMBER: _ClassVar[int]
    ITEM_MATRIX_ID_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_ITEM_FIELD_NUMBER: _ClassVar[int]
    chance: _common_pb2.Fraction
    currency_list: _currency_pb2.CraftCurrencyList
    item_matrix_id: int
    resolved_item: _item_pb2.ItemSnapshot
    def __init__(self, item_matrix_id: _Optional[int] = ..., chance: _Optional[_Union[_common_pb2.Fraction, _Mapping]] = ..., currency_list: _Optional[_Union[_currency_pb2.CraftCurrencyList, _Mapping]] = ..., resolved_item: _Optional[_Union[_item_pb2.ItemSnapshot, _Mapping]] = ...) -> None: ...

class JobEvent(_message.Message):
    __slots__ = ["error", "job_id", "progress", "result", "status"]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    RESULT_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    error: _common_pb2.Error
    job_id: str
    progress: JobProgress
    result: JobResult
    status: JobStatus
    def __init__(self, job_id: _Optional[str] = ..., status: _Optional[_Union[JobStatus, _Mapping]] = ..., progress: _Optional[_Union[JobProgress, _Mapping]] = ..., result: _Optional[_Union[JobResult, _Mapping]] = ..., error: _Optional[_Union[_common_pb2.Error, _Mapping]] = ...) -> None: ...

class JobProgress(_message.Message):
    __slots__ = ["percent", "phase", "ram_used_bytes", "routes_found"]
    PERCENT_FIELD_NUMBER: _ClassVar[int]
    PHASE_FIELD_NUMBER: _ClassVar[int]
    RAM_USED_BYTES_FIELD_NUMBER: _ClassVar[int]
    ROUTES_FOUND_FIELD_NUMBER: _ClassVar[int]
    percent: int
    phase: str
    ram_used_bytes: int
    routes_found: int
    def __init__(self, phase: _Optional[str] = ..., percent: _Optional[int] = ..., routes_found: _Optional[int] = ..., ram_used_bytes: _Optional[int] = ...) -> None: ...

class JobResult(_message.Message):
    __slots__ = ["group_results", "matrix_size", "path_results", "pretty_text"]
    GROUP_RESULTS_FIELD_NUMBER: _ClassVar[int]
    MATRIX_SIZE_FIELD_NUMBER: _ClassVar[int]
    PATH_RESULTS_FIELD_NUMBER: _ClassVar[int]
    PRETTY_TEXT_FIELD_NUMBER: _ClassVar[int]
    group_results: _containers.RepeatedCompositeFieldContainer[GroupAnalyzerResult]
    matrix_size: int
    path_results: _containers.RepeatedCompositeFieldContainer[PathAnalyzerResult]
    pretty_text: str
    def __init__(self, matrix_size: _Optional[int] = ..., path_results: _Optional[_Iterable[_Union[PathAnalyzerResult, _Mapping]]] = ..., group_results: _Optional[_Iterable[_Union[GroupAnalyzerResult, _Mapping]]] = ..., pretty_text: _Optional[str] = ...) -> None: ...

class JobStatus(_message.Message):
    __slots__ = ["created_at", "error", "finished_at", "job_id", "progress", "queue_position", "started_at", "state"]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    FINISHED_AT_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    QUEUE_POSITION_FIELD_NUMBER: _ClassVar[int]
    STARTED_AT_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    created_at: str
    error: _common_pb2.Error
    finished_at: str
    job_id: str
    progress: JobProgress
    queue_position: int
    started_at: str
    state: JobState
    def __init__(self, job_id: _Optional[str] = ..., state: _Optional[_Union[JobState, str]] = ..., queue_position: _Optional[int] = ..., progress: _Optional[_Union[JobProgress, _Mapping]] = ..., error: _Optional[_Union[_common_pb2.Error, _Mapping]] = ..., created_at: _Optional[str] = ..., started_at: _Optional[str] = ..., finished_at: _Optional[str] = ...) -> None: ...

class Limits(_message.Message):
    __slots__ = ["max_ram_in_bytes", "max_routes", "timeout_seconds"]
    MAX_RAM_IN_BYTES_FIELD_NUMBER: _ClassVar[int]
    MAX_ROUTES_FIELD_NUMBER: _ClassVar[int]
    TIMEOUT_SECONDS_FIELD_NUMBER: _ClassVar[int]
    max_ram_in_bytes: int
    max_routes: int
    timeout_seconds: int
    def __init__(self, max_routes: _Optional[int] = ..., max_ram_in_bytes: _Optional[int] = ..., timeout_seconds: _Optional[int] = ...) -> None: ...

class PathAnalyzerResult(_message.Message):
    __slots__ = ["lower_is_better", "preset", "routes", "unit_type"]
    LOWER_IS_BETTER_FIELD_NUMBER: _ClassVar[int]
    PRESET_FIELD_NUMBER: _ClassVar[int]
    ROUTES_FIELD_NUMBER: _ClassVar[int]
    UNIT_TYPE_FIELD_NUMBER: _ClassVar[int]
    lower_is_better: bool
    preset: _presets_pb2.StatisticAnalyzerPathPreset
    routes: _containers.RepeatedCompositeFieldContainer[ItemRoute]
    unit_type: str
    def __init__(self, preset: _Optional[_Union[_presets_pb2.StatisticAnalyzerPathPreset, str]] = ..., routes: _Optional[_Iterable[_Union[ItemRoute, _Mapping]]] = ..., lower_is_better: bool = ..., unit_type: _Optional[str] = ...) -> None: ...

class ResultOptions(_message.Message):
    __slots__ = ["include_pretty_strings", "include_route_snapshots", "top_n_pretty"]
    INCLUDE_PRETTY_STRINGS_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_ROUTE_SNAPSHOTS_FIELD_NUMBER: _ClassVar[int]
    TOP_N_PRETTY_FIELD_NUMBER: _ClassVar[int]
    include_pretty_strings: bool
    include_route_snapshots: bool
    top_n_pretty: int
    def __init__(self, include_pretty_strings: bool = ..., include_route_snapshots: bool = ..., top_n_pretty: _Optional[int] = ...) -> None: ...

class RouteChances(_message.Message):
    __slots__ = ["values"]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, values: _Optional[_Iterable[float]] = ...) -> None: ...

class SubmitJobRequest(_message.Message):
    __slots__ = ["group_analyzers", "league", "limits", "matrix_builder", "path_analyzers", "result_options", "start", "target"]
    GROUP_ANALYZERS_FIELD_NUMBER: _ClassVar[int]
    LEAGUE_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    MATRIX_BUILDER_FIELD_NUMBER: _ClassVar[int]
    PATH_ANALYZERS_FIELD_NUMBER: _ClassVar[int]
    RESULT_OPTIONS_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    group_analyzers: _containers.RepeatedScalarFieldContainer[_presets_pb2.StatisticAnalyzerCurrencyGroupPreset]
    league: str
    limits: Limits
    matrix_builder: _presets_pb2.MatrixBuilderPreset
    path_analyzers: _containers.RepeatedScalarFieldContainer[_presets_pb2.StatisticAnalyzerPathPreset]
    result_options: ResultOptions
    start: _item_pb2.ItemSnapshot
    target: _item_pb2.ItemSnapshot
    def __init__(self, league: _Optional[str] = ..., start: _Optional[_Union[_item_pb2.ItemSnapshot, _Mapping]] = ..., target: _Optional[_Union[_item_pb2.ItemSnapshot, _Mapping]] = ..., matrix_builder: _Optional[_Union[_presets_pb2.MatrixBuilderPreset, str]] = ..., path_analyzers: _Optional[_Iterable[_Union[_presets_pb2.StatisticAnalyzerPathPreset, str]]] = ..., group_analyzers: _Optional[_Iterable[_Union[_presets_pb2.StatisticAnalyzerCurrencyGroupPreset, str]]] = ..., limits: _Optional[_Union[Limits, _Mapping]] = ..., result_options: _Optional[_Union[ResultOptions, _Mapping]] = ...) -> None: ...

class SubmitJobResponse(_message.Message):
    __slots__ = ["job_id", "status"]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    status: JobStatus
    def __init__(self, job_id: _Optional[str] = ..., status: _Optional[_Union[JobStatus, _Mapping]] = ...) -> None: ...

class JobState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
