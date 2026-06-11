//! Shared pool-weight helpers used by the orb propagators.
//!
//! The duplicated `pool.retain(...)` filter blocks across propagators are
//! intentionally NOT unified yet: each has behavioral variations (homogen
//! tags, dextral/sinistral constraints, abyssal-replacement allowances,
//! tier-1 escape hatches) and a faulty extraction would silently change
//! matrix contents. Unify per-propagator with matrix-hash goldens when
//! touched next (see backend/README.md follow-ups).

use crate::domain::provider::item_info::AffixWeightTable;
use crate::domain::types::{AffixSpecifier, AffixTierLevelBoundsEnum};

/// Total weight of all surviving tiers in the pool (the probability
/// denominator). Verbatim extraction of the fold previously copy-pasted in
/// five propagators.
pub fn pool_total_weight(pool: &AffixWeightTable) -> u32 {
    pool.iter().fold(0u32, |a, (_, tier_meta)| {
        a + tier_meta
            .iter()
            .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone())
    })
}

/// Weight of the tiers of `affix` acceptable under its tier bounds
/// (Minimum: all tiers <= requested; Exact: exactly the requested tier).
/// Returns None when the affix is not in the pool or no tier qualifies.
pub fn acceptable_affix_weight(pool: &AffixWeightTable, affix: &AffixSpecifier) -> Option<u32> {
    let tiers = pool.get(&affix.affix)?;

    let weight = match affix.tier.bounds {
        AffixTierLevelBoundsEnum::Minimum => tiers
            .iter()
            .filter(|(tier, _)| **tier <= affix.tier.tier)
            .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone()),
        AffixTierLevelBoundsEnum::Exact => tiers
            .get(&affix.tier.tier)
            .map(|meta| meta.weight.get_raw_value().clone())
            .unwrap_or(0),
    };

    (weight > 0).then_some(weight)
}
