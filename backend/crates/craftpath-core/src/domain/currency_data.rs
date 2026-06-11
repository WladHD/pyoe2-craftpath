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

#[cfg(test)]
mod tests {
    use super::*;

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
