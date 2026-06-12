//! Static game data about currencies: minimum starting item levels for the
//! tiered orbs (game patch 0.5.0 values, MECHANICS.md sources). Display
//! names and the bone tables stay in `domain::currency::get_item_name` next
//! to the enum until a larger extraction is warranted.

use crate::domain::currency::CraftCurrencyEnum;
use crate::domain::types::ItemLevel;

/// The minimum item level a tiered orb requires for newly added mods
/// (`currencyModLevelFilter` in the CoE emulator). Returns 0 for untiered
/// currencies.
pub fn min_starting_item_level(currency: &CraftCurrencyEnum) -> ItemLevel {
    ItemLevel::from(match currency {
        // transmutation / augmentation: greater 44 (0.5.0: was 55), perfect 70
        CraftCurrencyEnum::OrbOfTransmutationNormal()
        | CraftCurrencyEnum::OrbOfAugmentationNormal() => 0,
        CraftCurrencyEnum::OrbOfTransmutationGreater()
        | CraftCurrencyEnum::OrbOfAugmentationGreater() => 44,
        CraftCurrencyEnum::OrbOfTransmutationPerfect()
        | CraftCurrencyEnum::OrbOfAugmentationPerfect() => 70,

        // regal / exalted / chaos: greater 35, perfect 50
        CraftCurrencyEnum::RegalOrbNormal()
        | CraftCurrencyEnum::ExaltedOrbNormal()
        | CraftCurrencyEnum::ChaosOrbNormal() => 0,
        CraftCurrencyEnum::RegalOrbGreater()
        | CraftCurrencyEnum::ExaltedOrbGreater()
        | CraftCurrencyEnum::ChaosOrbGreater() => 35,
        CraftCurrencyEnum::RegalOrbPerfect()
        | CraftCurrencyEnum::ExaltedOrbPerfect()
        | CraftCurrencyEnum::ChaosOrbPerfect() => 50,

        _ => 0,
    })
}

/// Coarse safety classification of a currency for chat-assistant warnings
/// ("what steps are dangerous / not reversible?"). Static worst-case per
/// currency; essence kinds differ (standard essences are additive, perfect/
/// alloy essences remove a mod), so `Essence` is classified conservatively
/// as `RemovalRisk` - resolve the kind via `EssenceDefinition` for a precise
/// answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CurrencyRiskClass {
    /// Only adds affixes or sockets; never removes progress.
    Safe,
    /// Removes or rerolls existing affixes; can lose wanted mods.
    DestructiveReroll,
    /// Removes a random affix; can brick the item by hitting a wanted mod.
    RemovalRisk,
    /// Locks state permanently (fractured affixes cannot be removed).
    Permanent,
    /// Corrupts the item; no further crafting afterwards.
    Irreversible,
}

impl CurrencyRiskClass {
    pub fn description(&self) -> &'static str {
        match self {
            CurrencyRiskClass::Safe => "additive; never removes progress",
            CurrencyRiskClass::DestructiveReroll => {
                "rerolls/removes existing affixes; wanted mods can be lost"
            }
            CurrencyRiskClass::RemovalRisk => {
                "removes a random affix; can hit a wanted mod and brick the item"
            }
            CurrencyRiskClass::Permanent => {
                "locks an affix permanently; the fracture cannot be undone"
            }
            CurrencyRiskClass::Irreversible => {
                "corrupts the item; no crafting is possible afterwards"
            }
        }
    }
}

/// Worst-case risk class per currency (see [`CurrencyRiskClass`] docs for
/// the essence caveat).
pub fn risk_class(currency: &CraftCurrencyEnum) -> CurrencyRiskClass {
    use CraftCurrencyEnum::*;
    match currency {
        VaalOrb() | OmenOfCorruption() => CurrencyRiskClass::Irreversible,
        FracturingOrb() => CurrencyRiskClass::Permanent,
        ChaosOrbNormal() | ChaosOrbGreater() | ChaosOrbPerfect() | DextralErasure()
        | SinistralErasure() | Whittling() => CurrencyRiskClass::DestructiveReroll,
        OrbOfAnnulment() | DextralAnnulment() | SinistralAnnulment() | Essence(_) => {
            CurrencyRiskClass::RemovalRisk
        }
        _ => CurrencyRiskClass::Safe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_classes() {
        assert_eq!(
            risk_class(&CraftCurrencyEnum::VaalOrb()),
            CurrencyRiskClass::Irreversible
        );
        assert_eq!(
            risk_class(&CraftCurrencyEnum::FracturingOrb()),
            CurrencyRiskClass::Permanent
        );
        assert_eq!(
            risk_class(&CraftCurrencyEnum::ChaosOrbNormal()),
            CurrencyRiskClass::DestructiveReroll
        );
        assert_eq!(
            risk_class(&CraftCurrencyEnum::OrbOfAnnulment()),
            CurrencyRiskClass::RemovalRisk
        );
        assert_eq!(
            risk_class(&CraftCurrencyEnum::ExaltedOrbNormal()),
            CurrencyRiskClass::Safe
        );
        assert_eq!(
            risk_class(&CraftCurrencyEnum::ArtificersOrb()),
            CurrencyRiskClass::Safe
        );
    }

    #[test]
    fn test_tier_thresholds_match_05_emulator() {
        let lvl = |c: &CraftCurrencyEnum| *min_starting_item_level(c).get_raw_value();
        assert_eq!(lvl(&CraftCurrencyEnum::OrbOfTransmutationGreater()), 44);
        assert_eq!(lvl(&CraftCurrencyEnum::OrbOfAugmentationGreater()), 44);
        assert_eq!(lvl(&CraftCurrencyEnum::OrbOfTransmutationPerfect()), 70);
        assert_eq!(lvl(&CraftCurrencyEnum::ExaltedOrbGreater()), 35);
        assert_eq!(lvl(&CraftCurrencyEnum::ExaltedOrbPerfect()), 50);
        assert_eq!(lvl(&CraftCurrencyEnum::RegalOrbGreater()), 35);
        assert_eq!(lvl(&CraftCurrencyEnum::VaalOrb()), 0);
    }
}
