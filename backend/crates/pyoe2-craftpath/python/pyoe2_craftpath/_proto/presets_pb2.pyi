from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor
MATRIX_BUILDER_PRESET_HAPPY_PATH: MatrixBuilderPreset
MATRIX_BUILDER_PRESET_UNSPECIFIED: MatrixBuilderPreset
STATISTIC_ANALYZER_CURRENCY_GROUP_PRESET_CURRENCY_GROUP_CHANCE: StatisticAnalyzerCurrencyGroupPreset
STATISTIC_ANALYZER_CURRENCY_GROUP_PRESET_CURRENCY_GROUP_CHANCE_MEMORY_HEAVY: StatisticAnalyzerCurrencyGroupPreset
STATISTIC_ANALYZER_CURRENCY_GROUP_PRESET_UNSPECIFIED: StatisticAnalyzerCurrencyGroupPreset
STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE: StatisticAnalyzerPathPreset
STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE_MEMORY_HEAVY: StatisticAnalyzerPathPreset
STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_COST: StatisticAnalyzerPathPreset
STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_EFFICIENCY: StatisticAnalyzerPathPreset
STATISTIC_ANALYZER_PATH_PRESET_UNSPECIFIED: StatisticAnalyzerPathPreset

class PresetInfo(_message.Message):
    __slots__ = ["description", "name"]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    description: str
    name: str
    def __init__(self, name: _Optional[str] = ..., description: _Optional[str] = ...) -> None: ...

class PresetList(_message.Message):
    __slots__ = ["group_analyzers", "matrix_builders", "path_analyzers"]
    GROUP_ANALYZERS_FIELD_NUMBER: _ClassVar[int]
    MATRIX_BUILDERS_FIELD_NUMBER: _ClassVar[int]
    PATH_ANALYZERS_FIELD_NUMBER: _ClassVar[int]
    group_analyzers: _containers.RepeatedCompositeFieldContainer[PresetInfo]
    matrix_builders: _containers.RepeatedCompositeFieldContainer[PresetInfo]
    path_analyzers: _containers.RepeatedCompositeFieldContainer[PresetInfo]
    def __init__(self, matrix_builders: _Optional[_Iterable[_Union[PresetInfo, _Mapping]]] = ..., path_analyzers: _Optional[_Iterable[_Union[PresetInfo, _Mapping]]] = ..., group_analyzers: _Optional[_Iterable[_Union[PresetInfo, _Mapping]]] = ...) -> None: ...

class MatrixBuilderPreset(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class StatisticAnalyzerPathPreset(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class StatisticAnalyzerCurrencyGroupPreset(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
