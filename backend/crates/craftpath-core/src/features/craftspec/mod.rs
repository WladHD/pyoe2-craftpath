//! EPPSSA craft-spec mini-DSL: a compact, deterministic notation for target
//! items ("I want a bow with EPPSSA").
//!
//! Grammar (case-insensitive, whitespace ignored):
//!
//! ```ebnf
//! spec      = slot+ ;                 (* 1..max_affix slots *)
//! slot      = letter , { qualifier } ;
//! letter    = "E" | "P" | "S" | "A" ; (* essence | prefix | suffix | abyss/desecrated *)
//! qualifier = tier | binding | fracture ;
//! tier      = digit+ , [ "x" ] ;      (* "P1" = prefix tier 1 or better; "x" = exact *)
//! binding   = "[" , ident , "]" ;     (* pin an affix: "[#1234]" by id, else fuzzy name *)
//! fracture  = "!" ;
//! ```
//!
//! The parser expands each slot into its candidate affix pool for the given
//! base item, reports the concrete-target fan-out, and, when every slot is
//! pinned to one affix, emits an exact [`ItemSnapshot`] target usable with
//! the existing exact-target calculation. Fuzzy (unpinned) targets are the
//! domain of the future template matcher; until then clients pin slots via
//! bindings.

use anyhow::{anyhow, bail, Result};
use serde::Serialize;

use crate::domain::{
    item::ItemSnapshot,
    provider::item_info::ItemInfoProvider,
    types::{
        AffixClassEnum, AffixId, AffixLocationEnum, AffixSpecifier, AffixTierConstraints,
        AffixTierLevel, AffixTierLevelBoundsEnum, BaseItemId, ItemLevel, ItemRarityEnum, THashMap,
        THashSet,
    },
};

/// One parsed slot of a craft spec.
#[derive(Clone, Debug, Serialize)]
pub struct CraftSpecSlot {
    pub index: u8,
    pub letter: char,
    /// `P`/`S` fix the location; `E`/`A` leave it to the pinned affix.
    pub location: Option<AffixLocationEnum>,
    pub class: AffixClassEnum,
    /// (tier level, exact) - `None` accepts any tier.
    pub tier: Option<(u8, bool)>,
    pub fractured: bool,
    pub pinned: Option<AffixId>,
    /// Eligible affix ids for this slot on the given base (the pinned affix
    /// only, when pinned).
    pub candidates: Vec<AffixId>,
}

/// Parse result: slots, fan-out estimate and (when fully pinned) the exact
/// target snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct CraftSpecTemplate {
    pub base_id: BaseItemId,
    pub item_level: ItemLevel,
    pub rarity: ItemRarityEnum,
    pub slots: Vec<CraftSpecSlot>,
    /// Product of candidate counts over all slots (saturating).
    pub estimated_concrete_targets: u128,
    /// Exact target when every slot is pinned; `None` otherwise.
    pub exact_target: Option<ItemSnapshot>,
}

struct RawSlot {
    letter: char,
    tier: Option<(u8, bool)>,
    fractured: bool,
    binding: Option<String>,
}

fn lex(spec: &str) -> Result<Vec<RawSlot>> {
    let mut slots: Vec<RawSlot> = Vec::new();
    let mut chars = spec.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            'e' | 'E' | 'p' | 'P' | 's' | 'S' | 'a' | 'A' => slots.push(RawSlot {
                letter: c.to_ascii_uppercase(),
                tier: None,
                fractured: false,
                binding: None,
            }),
            '0'..='9' => {
                let slot = slots
                    .last_mut()
                    .ok_or_else(|| anyhow!("tier digit '{c}' before any slot letter"))?;
                let mut value = c.to_digit(10).unwrap();
                while let Some(d) = chars.peek().and_then(|c| c.to_digit(10)) {
                    value = value * 10 + d;
                    chars.next();
                }
                let exact = matches!(chars.peek(), Some('x') | Some('X'));
                if exact {
                    chars.next();
                }
                if value == 0 || value > u8::MAX as u32 {
                    bail!("tier {value} out of range (1-255)");
                }
                if slot.tier.is_some() {
                    bail!("slot '{}' has two tier qualifiers", slot.letter);
                }
                slot.tier = Some((value as u8, exact));
            }
            '[' => {
                let slot = slots
                    .last_mut()
                    .ok_or_else(|| anyhow!("binding before any slot letter"))?;
                let mut ident = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(c) => ident.push(c),
                        None => bail!("unterminated '[' binding"),
                    }
                }
                if ident.trim().is_empty() {
                    bail!("empty '[]' binding");
                }
                if slot.binding.is_some() {
                    bail!("slot '{}' has two bindings", slot.letter);
                }
                slot.binding = Some(ident.trim().to_string());
            }
            '!' => {
                let slot = slots
                    .last_mut()
                    .ok_or_else(|| anyhow!("'!' before any slot letter"))?;
                slot.fractured = true;
            }
            other => bail!(
                "unexpected character '{other}' (expected E/P/S/A, digits, '[name]', '!' or whitespace)"
            ),
        }
    }

    if slots.is_empty() {
        bail!("empty craft spec");
    }
    Ok(slots)
}

/// Candidate affixes for a slot class on the base: `Base`-class candidates
/// come from the base mod pool with the slot's location, `Desecrated` from
/// the same pool, `Essence` from the essence tables that cover the base.
fn slot_candidates(
    class: &AffixClassEnum,
    location: &Option<AffixLocationEnum>,
    base_id: &BaseItemId,
    item_level: &ItemLevel,
    tier: &Option<(u8, bool)>,
    provider: &ItemInfoProvider,
) -> Result<Vec<AffixId>> {
    let mut out: Vec<AffixId> = Vec::new();

    if *class == AffixClassEnum::Essence {
        let mut seen: THashSet<AffixId> = THashSet::default();
        for def in provider.cache_essence_def.values() {
            if let Some(table) = def.base_tier_table.get(base_id) {
                for affix_id in table.keys() {
                    if seen.insert(affix_id.clone()) {
                        out.push(affix_id.clone());
                    }
                }
            }
        }
        out.sort();
        return Ok(out);
    }

    let pool = provider.lookup_base_item_mods(base_id)?;
    for (affix_id, tier_map) in pool.iter() {
        let def = provider.lookup_affix_definition(affix_id)?;
        if def.affix_class != *class {
            continue;
        }
        if let Some(loc) = location {
            if def.affix_location != *loc {
                continue;
            }
        }
        let reachable = tier_map.iter().any(|(t, meta)| {
            if meta.min_item_level > *item_level {
                return false;
            }
            match tier {
                None => true,
                Some((wanted, true)) => *t.get_raw_value() == *wanted,
                Some((wanted, false)) => *t.get_raw_value() <= *wanted,
            }
        });
        if reachable {
            out.push(affix_id.clone());
        }
    }
    out.sort();
    Ok(out)
}

/// Resolve a `[binding]` against the slot's candidates: `#123` by id,
/// anything else as a case-insensitive substring of the affix description.
/// Must match exactly one candidate.
fn resolve_binding(
    binding: &str,
    candidates: &[AffixId],
    provider: &ItemInfoProvider,
) -> Result<AffixId> {
    if let Some(id_str) = binding.strip_prefix('#') {
        let raw: u16 = id_str
            .parse()
            .map_err(|_| anyhow!("invalid affix id binding '#{id_str}'"))?;
        let id = AffixId::from(raw);
        if !candidates.contains(&id) {
            bail!("affix #{raw} is not eligible for this slot on this base");
        }
        return Ok(id);
    }

    let needle = binding.to_lowercase();
    let matches: Vec<&AffixId> = candidates
        .iter()
        .filter(|id| {
            provider
                .lookup_affix_definition(id)
                .map(|def| def.description_template.to_lowercase().contains(&needle))
                .unwrap_or(false)
        })
        .collect();

    match matches.len() {
        1 => Ok(matches[0].clone()),
        0 => bail!("binding '[{binding}]' matches no eligible affix for this slot"),
        n => {
            let names: Vec<String> = matches
                .iter()
                .take(5)
                .filter_map(|id| {
                    provider
                        .lookup_affix_definition(id)
                        .ok()
                        .map(|d| format!("#{} {}", id.get_raw_value(), d.description_template))
                })
                .collect();
            bail!(
                "binding '[{binding}]' is ambiguous ({n} matches): {}",
                names.join("; ")
            )
        }
    }
}

/// Worst (numerically largest) reachable tier of an affix in the base pool,
/// used as the "any tier" minimum bound on exact targets.
fn worst_reachable_tier(
    affix: &AffixId,
    base_id: &BaseItemId,
    item_level: &ItemLevel,
    provider: &ItemInfoProvider,
) -> Option<u8> {
    let pool = provider.lookup_base_item_mods(base_id).ok()?;
    pool.get(affix)?
        .iter()
        .filter(|(_, meta)| meta.min_item_level <= *item_level)
        .map(|(t, _)| *t.get_raw_value())
        .max()
}

/// Parse a craft spec against a base item. `bindings` pins slots by index
/// (0-based) in addition to inline `[...]` bindings.
pub fn parse_craft_spec(
    spec: &str,
    base_id: BaseItemId,
    item_level: ItemLevel,
    bindings: &THashMap<u8, AffixId>,
    provider: &ItemInfoProvider,
) -> Result<CraftSpecTemplate> {
    let raw_slots = lex(spec)?;

    let base_group_id = provider.lookup_base_group(&base_id)?;
    let base_group = provider.lookup_base_group_definition(&base_group_id)?;
    let side_cap = base_group.max_affix / 2;

    if raw_slots.len() > base_group.max_affix as usize {
        bail!(
            "spec has {} slots but base group '{}' allows at most {} affixes",
            raw_slots.len(),
            base_group.name_base_group,
            base_group.max_affix
        );
    }

    let mut slots: Vec<CraftSpecSlot> = Vec::new();
    for (index, raw) in raw_slots.into_iter().enumerate() {
        let (class, location) = match raw.letter {
            'P' => (AffixClassEnum::Base, Some(AffixLocationEnum::Prefix)),
            'S' => (AffixClassEnum::Base, Some(AffixLocationEnum::Suffix)),
            'E' => (AffixClassEnum::Essence, None),
            'A' => (AffixClassEnum::Desecrated, None),
            _ => unreachable!("lexer only emits EPSA"),
        };

        let candidates =
            slot_candidates(&class, &location, &base_id, &item_level, &raw.tier, provider)?;
        if candidates.is_empty() {
            bail!(
                "slot {} ('{}'): no eligible affix on this base{}",
                index + 1,
                raw.letter,
                raw.tier
                    .map(|(t, x)| format!(" at tier {}{}", t, if x { " exactly" } else { "+" }))
                    .unwrap_or_default()
            );
        }

        let pinned = match (&raw.binding, bindings.get(&(index as u8))) {
            (Some(_), Some(_)) => bail!(
                "slot {} is pinned both inline and via the bindings parameter",
                index + 1
            ),
            (Some(b), None) => Some(resolve_binding(b, &candidates, provider)?),
            (None, Some(id)) => {
                if !candidates.contains(id) {
                    bail!(
                        "bound affix #{} is not eligible for slot {}",
                        id.get_raw_value(),
                        index + 1
                    );
                }
                Some(id.clone())
            }
            (None, None) => None,
        };

        let candidates = match &pinned {
            Some(id) => vec![id.clone()],
            None => candidates,
        };

        slots.push(CraftSpecSlot {
            index: index as u8,
            letter: raw.letter,
            location,
            class,
            tier: raw.tier,
            fractured: raw.fractured,
            pinned,
            candidates,
        });
    }

    // per-side validation over the locations that are known up front
    let prefixes = slots
        .iter()
        .filter(|s| s.location == Some(AffixLocationEnum::Prefix))
        .count();
    let suffixes = slots
        .iter()
        .filter(|s| s.location == Some(AffixLocationEnum::Suffix))
        .count();
    if prefixes > side_cap as usize {
        bail!("{prefixes} prefix slots exceed the per-side cap of {side_cap}");
    }
    if suffixes > side_cap as usize {
        bail!("{suffixes} suffix slots exceed the per-side cap of {side_cap}");
    }

    let estimated_concrete_targets = slots
        .iter()
        .fold(1u128, |acc, s| acc.saturating_mul(s.candidates.len() as u128));

    let rarity = if slots.len() > 2 || !base_group.is_rare {
        if base_group.is_rare {
            ItemRarityEnum::Rare
        } else {
            ItemRarityEnum::Magic
        }
    } else {
        ItemRarityEnum::Rare
    };

    let exact_target = if slots.iter().all(|s| s.pinned.is_some()) {
        let mut affixes: THashSet<AffixSpecifier> = THashSet::default();
        for slot in &slots {
            let affix = slot.pinned.clone().unwrap();
            let (tier, bounds) = match slot.tier {
                Some((t, true)) => (t, AffixTierLevelBoundsEnum::Exact),
                Some((t, false)) => (t, AffixTierLevelBoundsEnum::Minimum),
                None => (
                    worst_reachable_tier(&affix, &base_id, &item_level, provider).unwrap_or(1),
                    AffixTierLevelBoundsEnum::Minimum,
                ),
            };
            affixes.insert(AffixSpecifier {
                affix,
                fractured: slot.fractured,
                tier: AffixTierConstraints {
                    tier: AffixTierLevel::from(tier),
                    bounds,
                },
            });
        }
        Some(ItemSnapshot {
            item_level: item_level.clone(),
            rarity: rarity.clone(),
            base_id: base_id.clone(),
            affixes,
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        })
    } else {
        None
    };

    Ok(CraftSpecTemplate {
        base_id,
        item_level,
        rarity,
        slots,
        estimated_concrete_targets,
        exact_target,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::inspect::tests::fixture_provider;

    #[test]
    fn test_parse_open_spec_counts_fanout() -> Result<()> {
        let provider = fixture_provider();
        let tpl = parse_craft_spec(
            "PSA",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )?;
        assert_eq!(tpl.slots.len(), 3);
        // P -> 1001 only; S -> 1002 only; A -> 1003 only
        assert_eq!(tpl.estimated_concrete_targets, 1);
        // all single-candidate but unpinned: no exact target
        assert!(tpl.exact_target.is_none());
        Ok(())
    }

    #[test]
    fn test_pinned_spec_builds_exact_target() -> Result<()> {
        let provider = fixture_provider();
        let tpl = parse_craft_spec(
            "P[phys]1 S[#1002]",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )?;
        let target = tpl.exact_target.expect("fully pinned spec yields target");
        assert_eq!(target.affixes.len(), 2);
        let phys = target
            .affixes
            .iter()
            .find(|a| a.affix == AffixId::from(1001u16))
            .unwrap();
        assert_eq!(*phys.tier.tier.get_raw_value(), 1);
        assert_eq!(phys.tier.bounds, AffixTierLevelBoundsEnum::Minimum);
        // unpinned tier on #1002 falls back to its worst reachable tier (1)
        let speed = target
            .affixes
            .iter()
            .find(|a| a.affix == AffixId::from(1002u16))
            .unwrap();
        assert_eq!(*speed.tier.tier.get_raw_value(), 1);
        Ok(())
    }

    #[test]
    fn test_bindings_parameter_and_validation_errors() -> Result<()> {
        let provider = fixture_provider();

        // external binding by slot index
        let mut bindings = THashMap::default();
        bindings.insert(0u8, AffixId::from(1001u16));
        let tpl = parse_craft_spec(
            "P",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &bindings,
            &provider,
        )?;
        assert!(tpl.exact_target.is_some());

        // too many slots for the base
        assert!(parse_craft_spec(
            "PPPPSSS",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )
        .is_err());

        // per-side cap: 4 prefixes on a 3-per-side base
        assert!(parse_craft_spec(
            "PPPP",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )
        .is_err());

        // unknown binding
        assert!(parse_craft_spec(
            "P[lightning]",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )
        .is_err());

        // exact tier qualifier parses
        let tpl = parse_craft_spec(
            "P[phys]2x",
            BaseItemId::from(20u16),
            ItemLevel::from(81u8),
            &THashMap::default(),
            &provider,
        )?;
        assert_eq!(tpl.slots[0].tier, Some((2, true)));
        let target = tpl.exact_target.unwrap();
        let spec = target.affixes.iter().next().unwrap();
        assert_eq!(spec.tier.bounds, AffixTierLevelBoundsEnum::Exact);
        Ok(())
    }
}
