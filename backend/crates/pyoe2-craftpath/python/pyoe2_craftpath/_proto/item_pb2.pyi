from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

AFFIX_TIER_LEVEL_BOUNDS_EXACT: AffixTierLevelBounds
AFFIX_TIER_LEVEL_BOUNDS_MINIMUM: AffixTierLevelBounds
AFFIX_TIER_LEVEL_BOUNDS_UNSPECIFIED: AffixTierLevelBounds
DESCRIPTOR: _descriptor.FileDescriptor
ITEM_RARITY_MAGIC: ItemRarity
ITEM_RARITY_NORMAL: ItemRarity
ITEM_RARITY_RARE: ItemRarity
ITEM_RARITY_UNIQUE: ItemRarity
ITEM_RARITY_UNSPECIFIED: ItemRarity

class AffixSpecifier(_message.Message):
    __slots__ = ["affix_id", "fractured", "tier"]
    AFFIX_ID_FIELD_NUMBER: _ClassVar[int]
    FRACTURED_FIELD_NUMBER: _ClassVar[int]
    TIER_FIELD_NUMBER: _ClassVar[int]
    affix_id: int
    fractured: bool
    tier: AffixTierConstraints
    def __init__(self, affix_id: _Optional[int] = ..., fractured: bool = ..., tier: _Optional[_Union[AffixTierConstraints, _Mapping]] = ...) -> None: ...

class AffixTierConstraints(_message.Message):
    __slots__ = ["bounds", "tier"]
    BOUNDS_FIELD_NUMBER: _ClassVar[int]
    TIER_FIELD_NUMBER: _ClassVar[int]
    bounds: AffixTierLevelBounds
    tier: int
    def __init__(self, tier: _Optional[int] = ..., bounds: _Optional[_Union[AffixTierLevelBounds, str]] = ...) -> None: ...

class ItemSnapshot(_message.Message):
    __slots__ = ["affixes", "allowed_sockets", "base_id", "corrupted", "item_level", "rarity", "sockets"]
    AFFIXES_FIELD_NUMBER: _ClassVar[int]
    ALLOWED_SOCKETS_FIELD_NUMBER: _ClassVar[int]
    BASE_ID_FIELD_NUMBER: _ClassVar[int]
    CORRUPTED_FIELD_NUMBER: _ClassVar[int]
    ITEM_LEVEL_FIELD_NUMBER: _ClassVar[int]
    RARITY_FIELD_NUMBER: _ClassVar[int]
    SOCKETS_FIELD_NUMBER: _ClassVar[int]
    affixes: _containers.RepeatedCompositeFieldContainer[AffixSpecifier]
    allowed_sockets: int
    base_id: int
    corrupted: bool
    item_level: int
    rarity: ItemRarity
    sockets: _containers.RepeatedCompositeFieldContainer[AffixSpecifier]
    def __init__(self, item_level: _Optional[int] = ..., rarity: _Optional[_Union[ItemRarity, str]] = ..., base_id: _Optional[int] = ..., affixes: _Optional[_Iterable[_Union[AffixSpecifier, _Mapping]]] = ..., corrupted: bool = ..., allowed_sockets: _Optional[int] = ..., sockets: _Optional[_Iterable[_Union[AffixSpecifier, _Mapping]]] = ...) -> None: ...

class ItemRarity(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class AffixTierLevelBounds(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
