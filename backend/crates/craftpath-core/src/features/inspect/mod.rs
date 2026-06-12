//! Read-only item inspection for chat assistants: which currencies are
//! legal on an item right now (with risk classes), and the one-step outcome
//! distribution of applying a single currency ("if I exalt-slam this, what
//! can I hit?").
//!
//! Deliberately independent from the happy-path matrix machinery: the
//! propagators answer "which moves lead toward a *target*", while this
//! module answers target-free questions from the raw affix pools. Legality
//! mirrors the propagator preconditions (rarity, affix capacity,
//! corruption) in simplified form and is documented as such.

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::domain::{
    currency::CraftCurrencyEnum,
    currency_data::{min_starting_item_level, risk_class, CurrencyRiskClass},
    item::ItemSnapshot,
    provider::item_info::ItemInfoProvider,
    types::{AffixClassEnum, AffixId, AffixLocationEnum, ItemRarityEnum, THashSet},
};

/// One currency that can legally be applied to an item right now.
#[derive(Clone, Debug, Serialize)]
pub struct LegalAction {
    pub currency: CraftCurrencyEnum,
    pub currency_name: String,
    pub risk: CurrencyRiskClass,
    pub risk_description: String,
    pub reason: String,
}

/// Whether an outcome adds a new affix or removes an existing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum OutcomeKind {
    Adds,
    Removes,
}

/// Chance of one tier of an outcome affix.
#[derive(Clone, Debug, Serialize)]
pub struct TierChance {
    pub tier: u8,
    pub weight: u32,
    pub min_item_level: u8,
}

/// One possible outcome of applying a currency once.
#[derive(Clone, Debug, Serialize)]
pub struct SimOutcome {
    pub kind: OutcomeKind,
    pub affix: AffixId,
    pub description: String,
    pub location: AffixLocationEnum,
    /// Probability of this affix being the one added/removed (sums to ~1.0
    /// over all outcomes of the same kind).
    pub chance: f64,
    pub tiers: Vec<TierChance>,
}

/// Full one-step distribution for (item, currency).
#[derive(Clone, Debug, Serialize)]
pub struct ActionSimulation {
    pub currency_name: String,
    pub risk: CurrencyRiskClass,
    pub outcomes: Vec<SimOutcome>,
    pub notes: Vec<String>,
}

/// Count existing prefix/suffix affixes (sockets and corruption implicits
/// never consume capacity).
pub fn affix_side_counts(
    snapshot: &ItemSnapshot,
    provider: &ItemInfoProvider,
) -> Result<(u8, u8)> {
    let mut prefixes = 0u8;
    let mut suffixes = 0u8;
    for spec in &snapshot.affixes {
        match provider.lookup_affix_definition(&spec.affix)?.affix_location {
            AffixLocationEnum::Prefix => prefixes += 1,
            AffixLocationEnum::Suffix => suffixes += 1,
            AffixLocationEnum::Socket | AffixLocationEnum::Corrupted => {}
        }
    }
    Ok((prefixes, suffixes))
}

fn action(
    currency: CraftCurrencyEnum,
    reason: impl Into<String>,
    provider: &ItemInfoProvider,
) -> LegalAction {
    let risk = risk_class(&currency);
    LegalAction {
        currency_name: currency.get_item_name(provider).to_string(),
        currency,
        risk,
        risk_description: risk.description().to_string(),
        reason: reason.into(),
    }
}

/// Tiered orbs are unusable when the item level is below the orb's mod
/// level filter (they would have no mods to roll).
fn orb_usable(currency: &CraftCurrencyEnum, snapshot: &ItemSnapshot) -> bool {
    min_starting_item_level(currency) <= snapshot.item_level
}

/// List the currencies that can legally be applied to the item right now,
/// each with a risk class and a short reason. Simplified rule mirror of the
/// propagator preconditions; omens are folded into their base orb's entry.
pub fn legal_actions(
    snapshot: &ItemSnapshot,
    provider: &ItemInfoProvider,
) -> Result<Vec<LegalAction>> {
    let mut out = Vec::new();

    if snapshot.corrupted {
        return Ok(out);
    }

    let base_group_id = provider.lookup_base_group(&snapshot.base_id)?;
    let base_group = provider.lookup_base_group_definition(&base_group_id)?;
    let (prefixes, suffixes) = affix_side_counts(snapshot, provider)?;
    let total = prefixes + suffixes;

    match snapshot.rarity {
        ItemRarityEnum::Normal if snapshot.affixes.is_empty() => {
            for c in [
                CraftCurrencyEnum::OrbOfTransmutationNormal(),
                CraftCurrencyEnum::OrbOfTransmutationGreater(),
                CraftCurrencyEnum::OrbOfTransmutationPerfect(),
            ] {
                if orb_usable(&c, snapshot) {
                    out.push(action(c, "Normal item: upgrades to Magic with one mod", provider));
                }
            }
        }
        ItemRarityEnum::Magic => {
            if total < 2 {
                for c in [
                    CraftCurrencyEnum::OrbOfAugmentationNormal(),
                    CraftCurrencyEnum::OrbOfAugmentationGreater(),
                    CraftCurrencyEnum::OrbOfAugmentationPerfect(),
                ] {
                    if orb_usable(&c, snapshot) {
                        out.push(action(c, "Magic item with an open mod slot: adds one mod", provider));
                    }
                }
            }
            if base_group.is_rare {
                for c in [
                    CraftCurrencyEnum::RegalOrbNormal(),
                    CraftCurrencyEnum::RegalOrbGreater(),
                    CraftCurrencyEnum::RegalOrbPerfect(),
                ] {
                    if orb_usable(&c, snapshot) {
                        out.push(action(c, "Magic item: upgrades to Rare and adds one mod", provider));
                    }
                }
            }
        }
        ItemRarityEnum::Rare => {
            if total < base_group.max_affix {
                for c in [
                    CraftCurrencyEnum::ExaltedOrbNormal(),
                    CraftCurrencyEnum::ExaltedOrbGreater(),
                    CraftCurrencyEnum::ExaltedOrbPerfect(),
                ] {
                    if orb_usable(&c, snapshot) {
                        out.push(action(
                            c,
                            "Rare item with an open affix slot: adds one mod (dextral/sinistral/homogenising omens steer the slot)",
                            provider,
                        ));
                    }
                }
                out.push(action(
                    CraftCurrencyEnum::Desecrator(snapshot.base_id.clone(), base_group_id.clone()),
                    "Rare item: adds a desecrated mod from the abyssal pool",
                    provider,
                ));
            }
            if total >= 1 {
                for c in [
                    CraftCurrencyEnum::ChaosOrbNormal(),
                    CraftCurrencyEnum::ChaosOrbGreater(),
                    CraftCurrencyEnum::ChaosOrbPerfect(),
                ] {
                    if orb_usable(&c, snapshot) {
                        out.push(action(
                            c,
                            "Rare item with mods: removes one mod and adds another",
                            provider,
                        ));
                    }
                }
                if snapshot.affixes.iter().any(|a| !a.fractured) {
                    out.push(action(
                        CraftCurrencyEnum::OrbOfAnnulment(),
                        "Rare item with removable mods: removes one random non-fractured mod",
                        provider,
                    ));
                }
                // placeholder essence id: get_item_name would look up a
                // concrete essence definition, so name it manually
                out.push(LegalAction {
                    currency: CraftCurrencyEnum::Essence(
                        crate::domain::types::EssenceId::from(0u16),
                    ),
                    currency_name: "Perfect/Alloy Essence (any)".to_string(),
                    risk: CurrencyRiskClass::RemovalRisk,
                    risk_description: CurrencyRiskClass::RemovalRisk.description().to_string(),
                    reason:
                        "Rare item: a Perfect/Alloy essence removes one mod and adds its guaranteed mod (pick a concrete essence)"
                            .to_string(),
                });
            }
            if total == 4 && snapshot.affixes.iter().all(|a| !a.fractured) {
                out.push(action(
                    CraftCurrencyEnum::FracturingOrb(),
                    "Rare item with exactly 4 unfractured mods: fractures one at random",
                    provider,
                ));
            }
        }
        _ => {}
    }

    if snapshot.allowed_sockets < base_group.max_sockets {
        out.push(action(
            CraftCurrencyEnum::ArtificersOrb(),
            "Item below its socket limit: adds a socket",
            provider,
        ));
    }

    out.push(action(
        CraftCurrencyEnum::VaalOrb(),
        "Uncorrupted item: corrupts with an unpredictable result",
        provider,
    ));

    Ok(out)
}

/// Compute the one-step outcome distribution of applying `currency` to the
/// item once. Supported: the additive orb families (transmutation,
/// augmentation, regal, exalted), desecration, a concrete essence, and the
/// orb of annulment. Other currencies (chaos rerolls, omen combinations,
/// vaal corruption) have compound outcomes and are answered by a full route
/// calculation instead.
pub fn simulate_action(
    snapshot: &ItemSnapshot,
    currency: &CraftCurrencyEnum,
    provider: &ItemInfoProvider,
) -> Result<ActionSimulation> {
    use CraftCurrencyEnum::*;

    if snapshot.corrupted {
        return Err(anyhow!("item is corrupted; no currency can be applied"));
    }

    let mut notes = Vec::new();
    let outcomes = match currency {
        OrbOfTransmutationNormal() | OrbOfTransmutationGreater() | OrbOfTransmutationPerfect() => {
            if snapshot.rarity != ItemRarityEnum::Normal || !snapshot.affixes.is_empty() {
                return Err(anyhow!("transmutation requires a Normal item without mods"));
            }
            additive_roll(snapshot, currency, provider, true, true)?
        }
        OrbOfAugmentationNormal() | OrbOfAugmentationGreater() | OrbOfAugmentationPerfect() => {
            if snapshot.rarity != ItemRarityEnum::Magic {
                return Err(anyhow!("augmentation requires a Magic item"));
            }
            let (p, s) = affix_side_counts(snapshot, provider)?;
            additive_roll(snapshot, currency, provider, p < 1, s < 1)?
        }
        RegalOrbNormal() | RegalOrbGreater() | RegalOrbPerfect() => {
            if snapshot.rarity != ItemRarityEnum::Magic {
                return Err(anyhow!("regal requires a Magic item"));
            }
            // the added mod rolls against the Rare per-side caps
            let base_group_id = provider.lookup_base_group(&snapshot.base_id)?;
            let cap = provider
                .lookup_base_group_definition(&base_group_id)?
                .max_affix
                / 2;
            let (p, s) = affix_side_counts(snapshot, provider)?;
            additive_roll(snapshot, currency, provider, p < cap, s < cap)?
        }
        ExaltedOrbNormal() | ExaltedOrbGreater() | ExaltedOrbPerfect() => {
            if snapshot.rarity != ItemRarityEnum::Rare {
                return Err(anyhow!("exalted requires a Rare item"));
            }
            let base_group_id = provider.lookup_base_group(&snapshot.base_id)?;
            let cap = provider
                .lookup_base_group_definition(&base_group_id)?
                .max_affix
                / 2;
            let (p, s) = affix_side_counts(snapshot, provider)?;
            additive_roll(snapshot, currency, provider, p < cap, s < cap)?
        }
        Desecrator(_, _) => {
            notes.push(
                "desecrated mod weights are not published; pool weights are taken from the dataset as-is"
                    .to_string(),
            );
            let base_group_id = provider.lookup_base_group(&snapshot.base_id)?;
            let cap = provider
                .lookup_base_group_definition(&base_group_id)?
                .max_affix
                / 2;
            let (p, s) = affix_side_counts(snapshot, provider)?;
            roll_from_pool(
                snapshot,
                provider,
                AffixClassEnum::Desecrated,
                p < cap,
                s < cap,
                0u8.into(),
            )?
        }
        Essence(essence_id) => {
            let def = provider.lookup_essence_definition(essence_id)?;
            notes.push(format!(
                "essence '{}' (kind {:?}{})",
                def.name_essence,
                def.kind,
                if def.corrupt { ", corrupts the item" } else { "" }
            ));
            let table = def.base_tier_table.get(&snapshot.base_id).ok_or_else(|| {
                anyhow!(
                    "essence '{}' has no mod table for this base item",
                    def.name_essence
                )
            })?;
            let n = table.len().max(1);
            table
                .keys()
                .map(|affix_id| {
                    let affix_def = provider.lookup_affix_definition(affix_id)?;
                    Ok(SimOutcome {
                        kind: OutcomeKind::Adds,
                        affix: affix_id.clone(),
                        description: affix_def.description_template.clone(),
                        location: affix_def.affix_location.clone(),
                        chance: 1.0 / n as f64,
                        tiers: Vec::new(),
                    })
                })
                .collect::<Result<Vec<_>>>()?
        }
        OrbOfAnnulment() => {
            let removable: Vec<_> = snapshot.affixes.iter().filter(|a| !a.fractured).collect();
            if removable.is_empty() {
                return Err(anyhow!("no removable (non-fractured) mods on the item"));
            }
            let n = removable.len();
            removable
                .into_iter()
                .map(|spec| {
                    let def = provider.lookup_affix_definition(&spec.affix)?;
                    Ok(SimOutcome {
                        kind: OutcomeKind::Removes,
                        affix: spec.affix.clone(),
                        description: def.description_template.clone(),
                        location: def.affix_location.clone(),
                        chance: 1.0 / n as f64,
                        tiers: Vec::new(),
                    })
                })
                .collect::<Result<Vec<_>>>()?
        }
        other => {
            return Err(anyhow!(
                "simulate_action does not support '{}'; supported: transmutation, augmentation, regal, exalted, desecration, a concrete essence, orb of annulment. Use a route calculation for compound currencies.",
                other.get_item_name(provider)
            ))
        }
    };

    let risk = risk_class(currency);
    Ok(ActionSimulation {
        currency_name: currency.get_item_name(provider).to_string(),
        risk,
        outcomes,
        notes,
    })
}

/// Roll distribution for the additive orb families over the Base-class pool.
fn additive_roll(
    snapshot: &ItemSnapshot,
    currency: &CraftCurrencyEnum,
    provider: &ItemInfoProvider,
    prefix_open: bool,
    suffix_open: bool,
) -> Result<Vec<SimOutcome>> {
    if !prefix_open && !suffix_open {
        return Err(anyhow!("no open affix slot on the item"));
    }
    roll_from_pool(
        snapshot,
        provider,
        AffixClassEnum::Base,
        prefix_open,
        suffix_open,
        min_starting_item_level(currency),
    )
}

/// Shared weighted-roll computation over the item's base mod pool: filters
/// by affix class, open location, exclusive groups already present on the
/// item, the item level, and the orb's mod level filter; normalizes by the
/// surviving total weight.
fn roll_from_pool(
    snapshot: &ItemSnapshot,
    provider: &ItemInfoProvider,
    class: AffixClassEnum,
    prefix_open: bool,
    suffix_open: bool,
    mod_level_filter: crate::domain::types::ItemLevel,
) -> Result<Vec<SimOutcome>> {
    let pool = provider.lookup_base_item_mods(&snapshot.base_id)?;

    let mut blocked: THashSet<String> = THashSet::default();
    for spec in &snapshot.affixes {
        blocked.extend(
            provider
                .lookup_affix_definition(&spec.affix)?
                .exlusive_groups
                .iter()
                .cloned(),
        );
    }

    struct Candidate {
        affix: AffixId,
        description: String,
        location: AffixLocationEnum,
        tiers: Vec<TierChance>,
        weight: u32,
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut total_weight = 0u64;

    for (affix_id, tier_map) in pool.iter() {
        let def = provider.lookup_affix_definition(affix_id)?;
        if def.affix_class != class {
            continue;
        }
        match def.affix_location {
            AffixLocationEnum::Prefix if !prefix_open => continue,
            AffixLocationEnum::Suffix if !suffix_open => continue,
            AffixLocationEnum::Socket | AffixLocationEnum::Corrupted => continue,
            _ => {}
        }
        if !def.exlusive_groups.is_disjoint(&blocked) {
            continue;
        }

        let mut tiers: Vec<TierChance> = tier_map
            .iter()
            .filter(|(_, meta)| {
                meta.min_item_level <= snapshot.item_level
                    && meta.min_item_level >= mod_level_filter
            })
            .map(|(tier, meta)| TierChance {
                tier: *tier.get_raw_value(),
                weight: *meta.weight.get_raw_value(),
                min_item_level: *meta.min_item_level.get_raw_value(),
            })
            .collect();
        tiers.sort_by_key(|t| t.tier);

        let weight: u32 = tiers.iter().map(|t| t.weight).sum();
        if weight == 0 {
            continue;
        }
        total_weight += weight as u64;
        candidates.push(Candidate {
            affix: affix_id.clone(),
            description: def.description_template.clone(),
            location: def.affix_location.clone(),
            tiers,
            weight,
        });
    }

    if total_weight == 0 {
        return Err(anyhow!(
            "no eligible mods in the pool for this item (class {:?})",
            class
        ));
    }

    let mut outcomes: Vec<SimOutcome> = candidates
        .into_iter()
        .map(|c| SimOutcome {
            kind: OutcomeKind::Adds,
            affix: c.affix,
            description: c.description,
            location: c.location,
            chance: c.weight as f64 / total_weight as f64,
            tiers: c.tiers,
        })
        .collect();
    outcomes.sort_by(|a, b| b.chance.partial_cmp(&a.chance).unwrap_or(std::cmp::Ordering::Equal));
    Ok(outcomes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::domain::provider::item_info::ItemInfoProvider;
    use crate::domain::types::{
        AffixDefinition, AffixSpecifier, AffixTierConstraints, AffixTierLevel,
        AffixTierLevelBoundsEnum, AffixTierLevelMeta, BaseGroupDefinition, BaseGroupId,
        BaseItemId, ItemLevel, THashMap, Weight,
    };

    pub(crate) fn fixture_provider() -> ItemInfoProvider {
        let mut cache_affix_def = THashMap::default();
        let affix = |desc: &str, class: AffixClassEnum, loc: AffixLocationEnum| AffixDefinition {
            exlusive_groups: THashSet::default(),
            tags: THashSet::default(),
            description_template: desc.to_string(),
            affix_class: class,
            affix_location: loc,
        };
        cache_affix_def.insert(
            AffixId::from(1001u16),
            affix("#% increased Physical Damage", AffixClassEnum::Base, AffixLocationEnum::Prefix),
        );
        cache_affix_def.insert(
            AffixId::from(1002u16),
            affix("#% increased Attack Speed", AffixClassEnum::Base, AffixLocationEnum::Suffix),
        );
        cache_affix_def.insert(
            AffixId::from(1003u16),
            affix("desecrated bone suffix", AffixClassEnum::Desecrated, AffixLocationEnum::Suffix),
        );

        let tier = |t: u8, w: u32, lvl: u8| {
            (
                AffixTierLevel::from(t),
                AffixTierLevelMeta {
                    weight: Weight::from(w),
                    min_item_level: ItemLevel::from(lvl),
                },
            )
        };

        let mut cache_item_affix_table = THashMap::default();
        let mut pool = THashMap::default();
        pool.insert(
            AffixId::from(1001u16),
            THashMap::from_iter(vec![tier(1, 100, 60), tier(2, 300, 30)]),
        );
        pool.insert(
            AffixId::from(1002u16),
            THashMap::from_iter(vec![tier(1, 100, 50)]),
        );
        pool.insert(
            AffixId::from(1003u16),
            THashMap::from_iter(vec![tier(1, 50, 65)]),
        );
        cache_item_affix_table.insert(BaseItemId::from(20u16), pool);

        let mut cache_base_group_table = THashMap::default();
        cache_base_group_table.insert(BaseItemId::from(20u16), BaseGroupId::from(7u16));

        let mut base_group_definition = THashMap::default();
        base_group_definition.insert(
            BaseGroupId::from(7u16),
            BaseGroupDefinition {
                name_base_group: "Bow".to_string(),
                max_affix: 6,
                max_sockets: 2,
                is_rare: true,
            },
        );

        ItemInfoProvider {
            cache_affix_def,
            cache_item_affix_table,
            cache_affix_essence_table: THashMap::default(),
            cache_essence_def: THashMap::default(),
            cache_base_group_table,
            base_group_definition,
        }
    }

    pub(crate) fn rare_item_with_phys() -> ItemSnapshot {
        let mut affixes = THashSet::default();
        affixes.insert(AffixSpecifier {
            affix: AffixId::from(1001u16),
            fractured: false,
            tier: AffixTierConstraints {
                tier: AffixTierLevel::from(2u8),
                bounds: AffixTierLevelBoundsEnum::Minimum,
            },
        });
        ItemSnapshot {
            item_level: ItemLevel::from(81u8),
            rarity: ItemRarityEnum::Rare,
            base_id: BaseItemId::from(20u16),
            affixes,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        }
    }

    #[test]
    fn test_exalt_simulation_rolls_open_suffix_only_pool() -> Result<()> {
        let provider = fixture_provider();
        let item = rare_item_with_phys();

        let sim = simulate_action(&item, &CraftCurrencyEnum::ExaltedOrbNormal(), &provider)?;
        // 1001 is present but prefixes are still open, so it could roll
        // again? No: same affix id stays in the pool only if not blocked by
        // exclusive groups; CoE pools list each affix once, an existing
        // affix id would re-roll its tier in game terms, but the dataset
        // models re-application as blocked via exclusive groups. Our fixture
        // has no exclusive groups, so both mods are eligible.
        assert_eq!(sim.outcomes.len(), 2);
        let total: f64 = sim.outcomes.iter().map(|o| o.chance).sum();
        assert!((total - 1.0).abs() < 1e-9);
        // phys: tiers 100+300=400, attack speed: 100 -> 0.8 / 0.2
        assert!((sim.outcomes[0].chance - 0.8).abs() < 1e-9);
        assert_eq!(sim.outcomes[0].tiers.len(), 2);
        Ok(())
    }

    #[test]
    fn test_simulation_respects_item_level_gate() -> Result<()> {
        let provider = fixture_provider();
        let mut item = rare_item_with_phys();
        item.item_level = ItemLevel::from(40u8);
        item.affixes = THashSet::default();

        let sim = simulate_action(&item, &CraftCurrencyEnum::ExaltedOrbNormal(), &provider)?;
        // at ilvl 40: phys t1 (60) and attack speed t1 (50) are gated out,
        // only phys t2 (30) remains
        assert_eq!(sim.outcomes.len(), 1);
        assert_eq!(sim.outcomes[0].tiers.len(), 1);
        assert_eq!(sim.outcomes[0].tiers[0].tier, 2);
        Ok(())
    }

    #[test]
    fn test_desecration_pool_and_annulment() -> Result<()> {
        let provider = fixture_provider();
        let item = rare_item_with_phys();

        let sim = simulate_action(
            &item,
            &CraftCurrencyEnum::Desecrator(BaseItemId::from(20u16), BaseGroupId::from(7u16)),
            &provider,
        )?;
        assert_eq!(sim.outcomes.len(), 1);
        assert_eq!(sim.outcomes[0].affix, AffixId::from(1003u16));

        let sim = simulate_action(&item, &CraftCurrencyEnum::OrbOfAnnulment(), &provider)?;
        assert_eq!(sim.outcomes.len(), 1);
        assert_eq!(sim.outcomes[0].kind, OutcomeKind::Removes);
        assert!((sim.outcomes[0].chance - 1.0).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_legal_actions_on_rare() -> Result<()> {
        let provider = fixture_provider();
        let item = rare_item_with_phys();

        let actions = legal_actions(&item, &provider)?;
        let names: Vec<&str> = actions.iter().map(|a| a.reason.as_str()).collect();
        assert!(!names.is_empty());
        // exalt legal (open slots), annul legal (1 removable), vaal always,
        // artificer (0 < 2 sockets), no transmutation/augmentation (Rare)
        assert!(actions
            .iter()
            .any(|a| matches!(a.currency, CraftCurrencyEnum::ExaltedOrbNormal())));
        assert!(actions
            .iter()
            .any(|a| matches!(a.currency, CraftCurrencyEnum::OrbOfAnnulment())));
        assert!(actions
            .iter()
            .any(|a| matches!(a.currency, CraftCurrencyEnum::VaalOrb())));
        assert!(actions
            .iter()
            .any(|a| matches!(a.currency, CraftCurrencyEnum::ArtificersOrb())));
        assert!(!actions
            .iter()
            .any(|a| matches!(a.currency, CraftCurrencyEnum::OrbOfTransmutationNormal())));

        // corrupted item: nothing is legal
        let mut corrupted = rare_item_with_phys();
        corrupted.corrupted = true;
        assert!(legal_actions(&corrupted, &provider)?.is_empty());
        Ok(())
    }
}
