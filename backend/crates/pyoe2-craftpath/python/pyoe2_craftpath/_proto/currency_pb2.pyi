from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

CRAFT_CURRENCY_KIND_ABYSSAL_ECHOES: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ARTIFICERS_ORB: CraftCurrencyKind
CRAFT_CURRENCY_KIND_CHAOS_ORB_GREATER: CraftCurrencyKind
CRAFT_CURRENCY_KIND_CHAOS_ORB_NORMAL: CraftCurrencyKind
CRAFT_CURRENCY_KIND_CHAOS_ORB_PERFECT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DESECRATOR: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DEXTRAL_ANNULMENT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DEXTRAL_CRYSTALLISATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DEXTRAL_ERASURE: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DEXTRAL_EXALTATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_DEXTRAL_NECROMANCY: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ESSENCE: CraftCurrencyKind
CRAFT_CURRENCY_KIND_EXALTED_ORB_GREATER: CraftCurrencyKind
CRAFT_CURRENCY_KIND_EXALTED_ORB_NORMAL: CraftCurrencyKind
CRAFT_CURRENCY_KIND_EXALTED_ORB_PERFECT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_FRACTURING_ORB: CraftCurrencyKind
CRAFT_CURRENCY_KIND_HOMOGENISING_CORONATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_HOMOGENISING_EXALTATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_OMEN_OF_CORRUPTION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_ANNULMENT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_AUGMENTATION_GREATER: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_AUGMENTATION_NORMAL: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_AUGMENTATION_PERFECT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_TRANSMUTATION_GREATER: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_TRANSMUTATION_NORMAL: CraftCurrencyKind
CRAFT_CURRENCY_KIND_ORB_OF_TRANSMUTATION_PERFECT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_REGAL_ORB_GREATER: CraftCurrencyKind
CRAFT_CURRENCY_KIND_REGAL_ORB_NORMAL: CraftCurrencyKind
CRAFT_CURRENCY_KIND_REGAL_ORB_PERFECT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_SINISTRAL_ANNULMENT: CraftCurrencyKind
CRAFT_CURRENCY_KIND_SINISTRAL_CRYSTALLISATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_SINISTRAL_ERASURE: CraftCurrencyKind
CRAFT_CURRENCY_KIND_SINISTRAL_EXALTATION: CraftCurrencyKind
CRAFT_CURRENCY_KIND_SINISTRAL_NECROMANCY: CraftCurrencyKind
CRAFT_CURRENCY_KIND_THE_BLACKBLOODED: CraftCurrencyKind
CRAFT_CURRENCY_KIND_THE_LIEGE: CraftCurrencyKind
CRAFT_CURRENCY_KIND_THE_SOVEREIGN: CraftCurrencyKind
CRAFT_CURRENCY_KIND_UNSPECIFIED: CraftCurrencyKind
CRAFT_CURRENCY_KIND_VAAL_ORB: CraftCurrencyKind
CRAFT_CURRENCY_KIND_WHITTLING: CraftCurrencyKind
DESCRIPTOR: _descriptor.FileDescriptor

class CraftCurrency(_message.Message):
    __slots__ = ["desecrator", "display_name", "essence_id", "kind"]
    DESECRATOR_FIELD_NUMBER: _ClassVar[int]
    DISPLAY_NAME_FIELD_NUMBER: _ClassVar[int]
    ESSENCE_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    desecrator: DesecratorPayload
    display_name: str
    essence_id: int
    kind: CraftCurrencyKind
    def __init__(self, kind: _Optional[_Union[CraftCurrencyKind, str]] = ..., desecrator: _Optional[_Union[DesecratorPayload, _Mapping]] = ..., essence_id: _Optional[int] = ..., display_name: _Optional[str] = ...) -> None: ...

class CraftCurrencyList(_message.Message):
    __slots__ = ["list"]
    LIST_FIELD_NUMBER: _ClassVar[int]
    list: _containers.RepeatedCompositeFieldContainer[CraftCurrency]
    def __init__(self, list: _Optional[_Iterable[_Union[CraftCurrency, _Mapping]]] = ...) -> None: ...

class DesecratorPayload(_message.Message):
    __slots__ = ["base_group_id", "base_item_id"]
    BASE_GROUP_ID_FIELD_NUMBER: _ClassVar[int]
    BASE_ITEM_ID_FIELD_NUMBER: _ClassVar[int]
    base_group_id: int
    base_item_id: int
    def __init__(self, base_item_id: _Optional[int] = ..., base_group_id: _Optional[int] = ...) -> None: ...

class CraftCurrencyKind(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
