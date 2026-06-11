//! Conversions between the craftpath-core domain types and the generated
//! `craftpath.v1` wire types.
//!
//! Domain → proto is infallible (results leaving the server). Proto → domain
//! is fallible (requests entering the server): out-of-range ids, unspecified
//! enums and inconsistent payloads are rejected with [`ConvertError`].

use crate::v1;

use craftpath_core::api::calculator::{GroupRoute, ItemRoute, ItemRouteNode};
use craftpath_core::api::currency::{CraftCurrencyEnum, CraftCurrencyList};
use craftpath_core::api::item::ItemSnapshot;
use craftpath_core::api::provider::item_info::ItemInfoProvider;
use craftpath_core::api::types::{
    AffixId, AffixSpecifier, AffixTierConstraints, AffixTierLevel, AffixTierLevelBoundsEnum,
    BaseGroupId, BaseItemId, EssenceId, ItemLevel, ItemRarityEnum, THashSet,
};
use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
use craftpath_core::utils::fraction_utils::Fraction;

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("field '{field}': value {value} exceeds the allowed maximum of {max}")]
    OutOfRange {
        field: &'static str,
        value: u64,
        max: u64,
    },
    #[error("field '{field}': enum value is unspecified or unknown")]
    UnspecifiedEnum { field: &'static str },
    #[error("craft currency kind '{kind}' requires the payload field '{payload}'")]
    MissingPayload { kind: &'static str, payload: &'static str },
}

fn check_u8(field: &'static str, value: u32) -> Result<u8, ConvertError> {
    u8::try_from(value).map_err(|_| ConvertError::OutOfRange {
        field,
        value: value as u64,
        max: u8::MAX as u64,
    })
}

fn check_u16(field: &'static str, value: u32) -> Result<u16, ConvertError> {
    u16::try_from(value).map_err(|_| ConvertError::OutOfRange {
        field,
        value: value as u64,
        max: u16::MAX as u64,
    })
}

// ---------------------------------------------------------------------------
// Fraction
// ---------------------------------------------------------------------------

impl From<&Fraction> for v1::Fraction {
    fn from(f: &Fraction) -> Self {
        v1::Fraction {
            num: f.num,
            den: f.den,
        }
    }
}

// ---------------------------------------------------------------------------
// Rarity / tier bounds enums
// ---------------------------------------------------------------------------

impl From<&ItemRarityEnum> for v1::ItemRarity {
    fn from(r: &ItemRarityEnum) -> Self {
        match r {
            ItemRarityEnum::Normal => v1::ItemRarity::Normal,
            ItemRarityEnum::Magic => v1::ItemRarity::Magic,
            ItemRarityEnum::Rare => v1::ItemRarity::Rare,
            ItemRarityEnum::Unique => v1::ItemRarity::Unique,
        }
    }
}

impl TryFrom<v1::ItemRarity> for ItemRarityEnum {
    type Error = ConvertError;

    fn try_from(r: v1::ItemRarity) -> Result<Self, Self::Error> {
        match r {
            v1::ItemRarity::Normal => Ok(ItemRarityEnum::Normal),
            v1::ItemRarity::Magic => Ok(ItemRarityEnum::Magic),
            v1::ItemRarity::Rare => Ok(ItemRarityEnum::Rare),
            v1::ItemRarity::Unique => Ok(ItemRarityEnum::Unique),
            v1::ItemRarity::Unspecified => {
                Err(ConvertError::UnspecifiedEnum { field: "rarity" })
            }
        }
    }
}

impl From<&AffixTierLevelBoundsEnum> for v1::AffixTierLevelBounds {
    fn from(b: &AffixTierLevelBoundsEnum) -> Self {
        match b {
            AffixTierLevelBoundsEnum::Exact => v1::AffixTierLevelBounds::Exact,
            AffixTierLevelBoundsEnum::Minimum => v1::AffixTierLevelBounds::Minimum,
        }
    }
}

impl TryFrom<v1::AffixTierLevelBounds> for AffixTierLevelBoundsEnum {
    type Error = ConvertError;

    fn try_from(b: v1::AffixTierLevelBounds) -> Result<Self, Self::Error> {
        match b {
            v1::AffixTierLevelBounds::Exact => Ok(AffixTierLevelBoundsEnum::Exact),
            v1::AffixTierLevelBounds::Minimum => Ok(AffixTierLevelBoundsEnum::Minimum),
            v1::AffixTierLevelBounds::Unspecified => Err(ConvertError::UnspecifiedEnum {
                field: "tier.bounds",
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Affix specifiers / item snapshots
// ---------------------------------------------------------------------------

impl From<&AffixSpecifier> for v1::AffixSpecifier {
    fn from(a: &AffixSpecifier) -> Self {
        v1::AffixSpecifier {
            affix_id: *a.affix.get_raw_value() as u32,
            fractured: a.fractured,
            tier: Some(v1::AffixTierConstraints {
                tier: *a.tier.tier.get_raw_value() as u32,
                bounds: v1::AffixTierLevelBounds::from(&a.tier.bounds) as i32,
            }),
        }
    }
}

impl TryFrom<&v1::AffixSpecifier> for AffixSpecifier {
    type Error = ConvertError;

    fn try_from(a: &v1::AffixSpecifier) -> Result<Self, Self::Error> {
        let tier = a.tier.as_ref().ok_or(ConvertError::UnspecifiedEnum {
            field: "affix.tier",
        })?;

        let bounds = v1::AffixTierLevelBounds::try_from(tier.bounds)
            .unwrap_or(v1::AffixTierLevelBounds::Unspecified);

        Ok(AffixSpecifier {
            affix: AffixId::from(check_u16("affix_id", a.affix_id)?),
            fractured: a.fractured,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(check_u8("tier.tier", tier.tier)?),
                bounds: AffixTierLevelBoundsEnum::try_from(bounds)?,
            },
        })
    }
}

impl From<&ItemSnapshot> for v1::ItemSnapshot {
    fn from(s: &ItemSnapshot) -> Self {
        v1::ItemSnapshot {
            item_level: *s.item_level.get_raw_value() as u32,
            rarity: v1::ItemRarity::from(&s.rarity) as i32,
            base_id: *s.base_id.get_raw_value() as u32,
            affixes: s.affixes.iter().map(v1::AffixSpecifier::from).collect(),
            corrupted: s.corrupted,
            allowed_sockets: s.allowed_sockets as u32,
            sockets: s.sockets.iter().map(v1::AffixSpecifier::from).collect(),
        }
    }
}

impl TryFrom<&v1::ItemSnapshot> for ItemSnapshot {
    type Error = ConvertError;

    fn try_from(s: &v1::ItemSnapshot) -> Result<Self, Self::Error> {
        let rarity = v1::ItemRarity::try_from(s.rarity).unwrap_or(v1::ItemRarity::Unspecified);

        let mut affixes: THashSet<AffixSpecifier> = THashSet::default();
        for a in &s.affixes {
            affixes.insert(AffixSpecifier::try_from(a)?);
        }

        let mut sockets: THashSet<AffixSpecifier> = THashSet::default();
        for a in &s.sockets {
            sockets.insert(AffixSpecifier::try_from(a)?);
        }

        Ok(ItemSnapshot {
            item_level: ItemLevel::from(check_u8("item_level", s.item_level)?),
            rarity: ItemRarityEnum::try_from(rarity)?,
            base_id: BaseItemId::from(check_u16("base_id", s.base_id)?),
            affixes,
            corrupted: s.corrupted,
            allowed_sockets: check_u8("allowed_sockets", s.allowed_sockets)?,
            sockets,
        })
    }
}

// ---------------------------------------------------------------------------
// Craft currencies
// ---------------------------------------------------------------------------

/// Convert a domain currency into its wire representation. When `item_info`
/// is provided the in-game `display_name` is resolved so clients do not need
/// any provider data.
pub fn craft_currency_to_proto(
    c: &CraftCurrencyEnum,
    item_info: Option<&ItemInfoProvider>,
) -> v1::CraftCurrency {
    use v1::CraftCurrencyKind as K;

    let (kind, desecrator, essence_id) = match c {
        CraftCurrencyEnum::OrbOfTransmutationNormal() => (K::OrbOfTransmutationNormal, None, None),
        CraftCurrencyEnum::OrbOfTransmutationGreater() => {
            (K::OrbOfTransmutationGreater, None, None)
        }
        CraftCurrencyEnum::OrbOfTransmutationPerfect() => {
            (K::OrbOfTransmutationPerfect, None, None)
        }
        CraftCurrencyEnum::OrbOfAugmentationNormal() => (K::OrbOfAugmentationNormal, None, None),
        CraftCurrencyEnum::OrbOfAugmentationGreater() => (K::OrbOfAugmentationGreater, None, None),
        CraftCurrencyEnum::OrbOfAugmentationPerfect() => (K::OrbOfAugmentationPerfect, None, None),
        CraftCurrencyEnum::RegalOrbNormal() => (K::RegalOrbNormal, None, None),
        CraftCurrencyEnum::RegalOrbGreater() => (K::RegalOrbGreater, None, None),
        CraftCurrencyEnum::RegalOrbPerfect() => (K::RegalOrbPerfect, None, None),
        CraftCurrencyEnum::ExaltedOrbNormal() => (K::ExaltedOrbNormal, None, None),
        CraftCurrencyEnum::ExaltedOrbGreater() => (K::ExaltedOrbGreater, None, None),
        CraftCurrencyEnum::ExaltedOrbPerfect() => (K::ExaltedOrbPerfect, None, None),
        CraftCurrencyEnum::OrbOfAnnulment() => (K::OrbOfAnnulment, None, None),
        CraftCurrencyEnum::ChaosOrbNormal() => (K::ChaosOrbNormal, None, None),
        CraftCurrencyEnum::ChaosOrbGreater() => (K::ChaosOrbGreater, None, None),
        CraftCurrencyEnum::ChaosOrbPerfect() => (K::ChaosOrbPerfect, None, None),
        CraftCurrencyEnum::ArtificersOrb() => (K::ArtificersOrb, None, None),
        CraftCurrencyEnum::VaalOrb() => (K::VaalOrb, None, None),
        CraftCurrencyEnum::OmenOfCorruption() => (K::OmenOfCorruption, None, None),
        CraftCurrencyEnum::FracturingOrb() => (K::FracturingOrb, None, None),
        CraftCurrencyEnum::Desecrator(base_item_id, base_group_id) => (
            K::Desecrator,
            Some(v1::DesecratorPayload {
                base_item_id: *base_item_id.get_raw_value() as u32,
                base_group_id: *base_group_id.get_raw_value() as u32,
            }),
            None,
        ),
        CraftCurrencyEnum::AbyssalEchoes() => (K::AbyssalEchoes, None, None),
        CraftCurrencyEnum::TheBlackblooded() => (K::TheBlackblooded, None, None),
        CraftCurrencyEnum::TheSovereign() => (K::TheSovereign, None, None),
        CraftCurrencyEnum::TheLiege() => (K::TheLiege, None, None),
        CraftCurrencyEnum::DextralNecromancy() => (K::DextralNecromancy, None, None),
        CraftCurrencyEnum::SinistralNecromancy() => (K::SinistralNecromancy, None, None),
        CraftCurrencyEnum::HomogenisingCoronation() => (K::HomogenisingCoronation, None, None),
        CraftCurrencyEnum::HomogenisingExaltation() => (K::HomogenisingExaltation, None, None),
        CraftCurrencyEnum::DextralExaltation() => (K::DextralExaltation, None, None),
        CraftCurrencyEnum::SinistralExaltation() => (K::SinistralExaltation, None, None),
        CraftCurrencyEnum::DextralAnnulment() => (K::DextralAnnulment, None, None),
        CraftCurrencyEnum::SinistralAnnulment() => (K::SinistralAnnulment, None, None),
        CraftCurrencyEnum::DextralErasure() => (K::DextralErasure, None, None),
        CraftCurrencyEnum::SinistralErasure() => (K::SinistralErasure, None, None),
        CraftCurrencyEnum::Whittling() => (K::Whittling, None, None),
        CraftCurrencyEnum::OmenOfGreaterExaltation() => (K::OmenOfGreaterExaltation, None, None),
        CraftCurrencyEnum::OmenOfLight() => (K::OmenOfLight, None, None),
        CraftCurrencyEnum::Essence(essence_id) => (
            K::Essence,
            None,
            Some(*essence_id.get_raw_value() as u32),
        ),
        CraftCurrencyEnum::DextralCrystallisation() => (K::DextralCrystallisation, None, None),
        CraftCurrencyEnum::SinistralCrystallisation() => (K::SinistralCrystallisation, None, None),
    };

    v1::CraftCurrency {
        kind: kind as i32,
        desecrator,
        essence_id,
        display_name: item_info
            .map(|info| c.get_item_name(info).to_string())
            .unwrap_or_default(),
    }
}

impl TryFrom<&v1::CraftCurrency> for CraftCurrencyEnum {
    type Error = ConvertError;

    fn try_from(c: &v1::CraftCurrency) -> Result<Self, Self::Error> {
        use v1::CraftCurrencyKind as K;

        let kind = v1::CraftCurrencyKind::try_from(c.kind).unwrap_or(K::Unspecified);

        Ok(match kind {
            K::Unspecified => {
                return Err(ConvertError::UnspecifiedEnum {
                    field: "currency.kind",
                });
            }
            K::OrbOfTransmutationNormal => CraftCurrencyEnum::OrbOfTransmutationNormal(),
            K::OrbOfTransmutationGreater => CraftCurrencyEnum::OrbOfTransmutationGreater(),
            K::OrbOfTransmutationPerfect => CraftCurrencyEnum::OrbOfTransmutationPerfect(),
            K::OrbOfAugmentationNormal => CraftCurrencyEnum::OrbOfAugmentationNormal(),
            K::OrbOfAugmentationGreater => CraftCurrencyEnum::OrbOfAugmentationGreater(),
            K::OrbOfAugmentationPerfect => CraftCurrencyEnum::OrbOfAugmentationPerfect(),
            K::RegalOrbNormal => CraftCurrencyEnum::RegalOrbNormal(),
            K::RegalOrbGreater => CraftCurrencyEnum::RegalOrbGreater(),
            K::RegalOrbPerfect => CraftCurrencyEnum::RegalOrbPerfect(),
            K::ExaltedOrbNormal => CraftCurrencyEnum::ExaltedOrbNormal(),
            K::ExaltedOrbGreater => CraftCurrencyEnum::ExaltedOrbGreater(),
            K::ExaltedOrbPerfect => CraftCurrencyEnum::ExaltedOrbPerfect(),
            K::OrbOfAnnulment => CraftCurrencyEnum::OrbOfAnnulment(),
            K::ChaosOrbNormal => CraftCurrencyEnum::ChaosOrbNormal(),
            K::ChaosOrbGreater => CraftCurrencyEnum::ChaosOrbGreater(),
            K::ChaosOrbPerfect => CraftCurrencyEnum::ChaosOrbPerfect(),
            K::ArtificersOrb => CraftCurrencyEnum::ArtificersOrb(),
            K::VaalOrb => CraftCurrencyEnum::VaalOrb(),
            K::OmenOfCorruption => CraftCurrencyEnum::OmenOfCorruption(),
            K::FracturingOrb => CraftCurrencyEnum::FracturingOrb(),
            K::Desecrator => {
                let payload = c.desecrator.as_ref().ok_or(ConvertError::MissingPayload {
                    kind: "DESECRATOR",
                    payload: "desecrator",
                })?;
                CraftCurrencyEnum::Desecrator(
                    BaseItemId::from(check_u16("desecrator.base_item_id", payload.base_item_id)?),
                    BaseGroupId::from(check_u16(
                        "desecrator.base_group_id",
                        payload.base_group_id,
                    )?),
                )
            }
            K::AbyssalEchoes => CraftCurrencyEnum::AbyssalEchoes(),
            K::TheBlackblooded => CraftCurrencyEnum::TheBlackblooded(),
            K::TheSovereign => CraftCurrencyEnum::TheSovereign(),
            K::TheLiege => CraftCurrencyEnum::TheLiege(),
            K::DextralNecromancy => CraftCurrencyEnum::DextralNecromancy(),
            K::SinistralNecromancy => CraftCurrencyEnum::SinistralNecromancy(),
            K::HomogenisingCoronation => CraftCurrencyEnum::HomogenisingCoronation(),
            K::HomogenisingExaltation => CraftCurrencyEnum::HomogenisingExaltation(),
            K::DextralExaltation => CraftCurrencyEnum::DextralExaltation(),
            K::SinistralExaltation => CraftCurrencyEnum::SinistralExaltation(),
            K::DextralAnnulment => CraftCurrencyEnum::DextralAnnulment(),
            K::SinistralAnnulment => CraftCurrencyEnum::SinistralAnnulment(),
            K::DextralErasure => CraftCurrencyEnum::DextralErasure(),
            K::SinistralErasure => CraftCurrencyEnum::SinistralErasure(),
            K::Whittling => CraftCurrencyEnum::Whittling(),
            K::Essence => {
                let essence_id = c.essence_id.ok_or(ConvertError::MissingPayload {
                    kind: "ESSENCE",
                    payload: "essence_id",
                })?;
                CraftCurrencyEnum::Essence(EssenceId::from(check_u16("essence_id", essence_id)?))
            }
            K::DextralCrystallisation => CraftCurrencyEnum::DextralCrystallisation(),
            K::SinistralCrystallisation => CraftCurrencyEnum::SinistralCrystallisation(),
            K::OmenOfGreaterExaltation => CraftCurrencyEnum::OmenOfGreaterExaltation(),
            K::OmenOfLight => CraftCurrencyEnum::OmenOfLight(),
        })
    }
}

pub fn craft_currency_list_to_proto(
    list: &CraftCurrencyList,
    item_info: Option<&ItemInfoProvider>,
) -> v1::CraftCurrencyList {
    v1::CraftCurrencyList {
        list: list
            .list
            .iter()
            .map(|c| craft_currency_to_proto(c, item_info))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Presets
// ---------------------------------------------------------------------------

impl From<&MatrixBuilderPreset> for v1::MatrixBuilderPreset {
    fn from(p: &MatrixBuilderPreset) -> Self {
        match p {
            MatrixBuilderPreset::HappyPathMatrixBuilder => v1::MatrixBuilderPreset::HappyPath,
        }
    }
}

impl TryFrom<v1::MatrixBuilderPreset> for MatrixBuilderPreset {
    type Error = ConvertError;

    fn try_from(p: v1::MatrixBuilderPreset) -> Result<Self, Self::Error> {
        match p {
            v1::MatrixBuilderPreset::HappyPath => Ok(MatrixBuilderPreset::HappyPathMatrixBuilder),
            v1::MatrixBuilderPreset::Unspecified => Err(ConvertError::UnspecifiedEnum {
                field: "matrix_builder",
            }),
        }
    }
}

impl From<&StatisticAnalyzerPathPreset> for v1::StatisticAnalyzerPathPreset {
    fn from(p: &StatisticAnalyzerPathPreset) -> Self {
        match p {
            StatisticAnalyzerPathPreset::UniquePathChance => {
                v1::StatisticAnalyzerPathPreset::UniquePathChance
            }
            StatisticAnalyzerPathPreset::UniquePathEfficiency => {
                v1::StatisticAnalyzerPathPreset::UniquePathEfficiency
            }
            StatisticAnalyzerPathPreset::UniquePathCost => {
                v1::StatisticAnalyzerPathPreset::UniquePathCost
            }
            StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy => {
                v1::StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy
            }
        }
    }
}

impl TryFrom<v1::StatisticAnalyzerPathPreset> for StatisticAnalyzerPathPreset {
    type Error = ConvertError;

    fn try_from(p: v1::StatisticAnalyzerPathPreset) -> Result<Self, Self::Error> {
        match p {
            v1::StatisticAnalyzerPathPreset::UniquePathChance => {
                Ok(StatisticAnalyzerPathPreset::UniquePathChance)
            }
            v1::StatisticAnalyzerPathPreset::UniquePathEfficiency => {
                Ok(StatisticAnalyzerPathPreset::UniquePathEfficiency)
            }
            v1::StatisticAnalyzerPathPreset::UniquePathCost => {
                Ok(StatisticAnalyzerPathPreset::UniquePathCost)
            }
            v1::StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy => {
                Ok(StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy)
            }
            v1::StatisticAnalyzerPathPreset::Unspecified => Err(ConvertError::UnspecifiedEnum {
                field: "path_analyzers",
            }),
        }
    }
}

impl From<&StatisticAnalyzerCurrencyGroupPreset> for v1::StatisticAnalyzerCurrencyGroupPreset {
    fn from(p: &StatisticAnalyzerCurrencyGroupPreset) -> Self {
        match p {
            StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance => {
                v1::StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance
            }
            StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy => {
                v1::StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy
            }
        }
    }
}

impl TryFrom<v1::StatisticAnalyzerCurrencyGroupPreset> for StatisticAnalyzerCurrencyGroupPreset {
    type Error = ConvertError;

    fn try_from(p: v1::StatisticAnalyzerCurrencyGroupPreset) -> Result<Self, Self::Error> {
        match p {
            v1::StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance => {
                Ok(StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance)
            }
            v1::StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy => {
                Ok(StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy)
            }
            v1::StatisticAnalyzerCurrencyGroupPreset::Unspecified => {
                Err(ConvertError::UnspecifiedEnum {
                    field: "group_analyzers",
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Routes (results, domain → proto only)
// ---------------------------------------------------------------------------

pub fn item_route_node_to_proto(
    node: &ItemRouteNode,
    item_info: Option<&ItemInfoProvider>,
) -> v1::ItemRouteNode {
    v1::ItemRouteNode {
        item_matrix_id: node.item_matrix_id,
        chance: Some(v1::Fraction::from(&node.chance)),
        currency_list: Some(craft_currency_list_to_proto(&node.currency_list, item_info)),
        resolved_item: None,
    }
}

pub fn item_route_to_proto(
    route: &ItemRoute,
    item_info: Option<&ItemInfoProvider>,
) -> v1::ItemRoute {
    v1::ItemRoute {
        route: route
            .route
            .iter()
            .map(|n| item_route_node_to_proto(n, item_info))
            .collect(),
        weight: *route.weight.get_raw_value(),
        chance: *route.chance.get_raw_value(),
        pretty: None,
    }
}

pub fn group_route_to_proto(
    group: &GroupRoute,
    item_info: Option<&ItemInfoProvider>,
) -> v1::GroupRoute {
    v1::GroupRoute {
        group: group
            .group
            .iter()
            .map(|l| craft_currency_list_to_proto(l, item_info))
            .collect(),
        weight: *group.weight.get_raw_value(),
        unique_route_weights: group
            .unique_route_weights
            .iter()
            .map(|chances| v1::RouteChances {
                values: chances.iter().map(|c| *c.get_raw_value()).collect(),
            })
            .collect(),
        chance: *group.chance.get_raw_value(),
        amount_subpaths: *group.amount_subpaths.get_raw_value(),
        pretty: None,
    }
}
