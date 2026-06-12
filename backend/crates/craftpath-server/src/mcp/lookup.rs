//! Pure helpers behind the sync MCP lookup tools: human input -> domain
//! values (currency names, affix locations/classes). Kept out of the tool
//! handlers so they are unit-testable without an MCP transport.

use anyhow::{anyhow, bail, Result};

use craftpath_core::prelude::{CraftCurrencyEnum, ItemInfoProvider, ItemSnapshot};
use craftpath_core::api::types::{AffixClassEnum, AffixLocationEnum};

/// All currencies with provider-independent display names (everything
/// except `Essence`/`Desecrator`, which need context).
pub fn static_currencies() -> Vec<CraftCurrencyEnum> {
    use CraftCurrencyEnum::*;
    vec![
        OrbOfTransmutationNormal(),
        OrbOfTransmutationGreater(),
        OrbOfTransmutationPerfect(),
        OrbOfAugmentationNormal(),
        OrbOfAugmentationGreater(),
        OrbOfAugmentationPerfect(),
        RegalOrbNormal(),
        RegalOrbGreater(),
        RegalOrbPerfect(),
        ExaltedOrbNormal(),
        ExaltedOrbGreater(),
        ExaltedOrbPerfect(),
        ChaosOrbNormal(),
        ChaosOrbGreater(),
        ChaosOrbPerfect(),
        OrbOfAnnulment(),
        ArtificersOrb(),
        VaalOrb(),
        FracturingOrb(),
        OmenOfCorruption(),
        AbyssalEchoes(),
        OmenOfLight(),
        OmenOfGreaterExaltation(),
        TheBlackblooded(),
        TheSovereign(),
        TheLiege(),
        DextralNecromancy(),
        SinistralNecromancy(),
        HomogenisingCoronation(),
        HomogenisingExaltation(),
        DextralExaltation(),
        SinistralExaltation(),
        DextralAnnulment(),
        SinistralAnnulment(),
        DextralCrystallisation(),
        SinistralCrystallisation(),
        DextralErasure(),
        SinistralErasure(),
        Whittling(),
    ]
}

/// Resolve a human currency name ("exalted orb", "greater chaos",
/// "desecrate", an essence name) against the static list, the desecration
/// bones for the item's base, and the league's essence table. Exact name
/// match wins; otherwise a unique substring match is required.
pub fn resolve_currency(
    input: &str,
    snapshot: &ItemSnapshot,
    items: &ItemInfoProvider,
) -> Result<CraftCurrencyEnum> {
    let needle = input.trim().to_lowercase();
    if needle.is_empty() {
        bail!("empty currency name");
    }

    let statics = static_currencies();

    // names of the static variants never consult the provider's essence table
    if let Some(exact) = statics
        .iter()
        .find(|c| c.get_item_name(items).to_lowercase() == needle)
    {
        return Ok(exact.clone());
    }

    if needle.contains("desecrat") || needle.contains("bone") {
        let group = items.lookup_base_group(&snapshot.base_id)?;
        return Ok(CraftCurrencyEnum::Desecrator(
            snapshot.base_id.clone(),
            group,
        ));
    }

    let mut matches: Vec<CraftCurrencyEnum> = statics
        .iter()
        .filter(|c| c.get_item_name(items).to_lowercase().contains(&needle))
        .cloned()
        .collect();

    for (essence_id, def) in items.cache_essence_def.iter() {
        if def.name_essence.to_lowercase().contains(&needle) {
            matches.push(CraftCurrencyEnum::Essence(essence_id.clone()));
        }
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => bail!(
            "unknown currency '{input}'; use a name like 'Exalted Orb', 'Greater Chaos Orb', 'desecrate' or an essence name"
        ),
        n => {
            let names: Vec<String> = matches
                .iter()
                .take(6)
                .map(|c| c.get_item_name(items).to_string())
                .collect();
            bail!(
                "currency '{input}' is ambiguous ({n} matches): {}",
                names.join(", ")
            )
        }
    }
}

pub fn parse_location(input: &str) -> Result<AffixLocationEnum> {
    match input.trim().to_lowercase().as_str() {
        "prefix" => Ok(AffixLocationEnum::Prefix),
        "suffix" => Ok(AffixLocationEnum::Suffix),
        "socket" => Ok(AffixLocationEnum::Socket),
        other => Err(anyhow!(
            "unknown location '{other}' (use prefix|suffix|socket)"
        )),
    }
}

pub fn parse_affix_class(input: &str) -> Result<AffixClassEnum> {
    match input.trim().to_lowercase().as_str() {
        "base" => Ok(AffixClassEnum::Base),
        "essence" => Ok(AffixClassEnum::Essence),
        "desecrated" | "abyss" | "abyssal" => Ok(AffixClassEnum::Desecrated),
        other => Err(anyhow!(
            "unknown affix class '{other}' (use base|essence|desecrated)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftpath_core::api::types::{
        BaseGroupDefinition, BaseGroupId, BaseItemId, ItemLevel, ItemRarityEnum, THashMap,
        THashSet,
    };

    fn provider_with_base() -> ItemInfoProvider {
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
            cache_affix_def: THashMap::default(),
            cache_item_affix_table: THashMap::default(),
            cache_affix_essence_table: THashMap::default(),
            cache_essence_def: THashMap::default(),
            cache_base_group_table,
            base_group_definition,
        }
    }

    fn snapshot() -> ItemSnapshot {
        ItemSnapshot {
            item_level: ItemLevel::from(81u8),
            rarity: ItemRarityEnum::Rare,
            base_id: BaseItemId::from(20u16),
            affixes: THashSet::default(),
            corrupted: false,
            allowed_sockets: 0,
            sockets: THashSet::default(),
        }
    }

    #[test]
    fn test_resolve_currency_names() -> Result<()> {
        let items = provider_with_base();
        let snap = snapshot();

        assert!(matches!(
            resolve_currency("Exalted Orb", &snap, &items)?,
            CraftCurrencyEnum::ExaltedOrbNormal()
        ));
        assert!(matches!(
            resolve_currency("greater chaos orb", &snap, &items)?,
            CraftCurrencyEnum::ChaosOrbGreater()
        ));
        assert!(matches!(
            resolve_currency("desecrate", &snap, &items)?,
            CraftCurrencyEnum::Desecrator(_, _)
        ));
        // ambiguous: "chaos" matches three orbs
        assert!(resolve_currency("chaos", &snap, &items).is_err());
        assert!(resolve_currency("no-such-orb", &snap, &items).is_err());
        Ok(())
    }
}
