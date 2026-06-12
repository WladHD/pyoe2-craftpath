//! Curated "what is currently good" meta archetypes - the always-available
//! v1 data source behind the get_meta_items tool. The embedded JSON ships
//! with the binary; `META_ARCHETYPES_PATH` overrides it with an operator-
//! maintained file (same schema). A live provider (poe.ninja builds /
//! ladder) can replace this later without changing the tool surface.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const EMBEDDED: &str = include_str!("../data/meta_archetypes.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetaArchetype {
    pub slot: String,
    pub item_class: String,
    pub char_classes: Vec<String>,
    pub level_bracket: String,
    pub archetype: String,
    pub affixes: Vec<String>,
    #[serde(default)]
    pub craft_spec: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetaCatalog {
    pub source: String,
    pub updated: String,
    pub game_patch: String,
    pub disclaimer: String,
    pub archetypes: Vec<MetaArchetype>,
}

/// Load the catalog from `META_ARCHETYPES_PATH` if set, otherwise the
/// embedded copy.
pub fn load_catalog() -> Result<MetaCatalog> {
    match std::env::var("META_ARCHETYPES_PATH") {
        Ok(path) if !path.trim().is_empty() => {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("could not read META_ARCHETYPES_PATH '{path}'"))?;
            serde_json::from_str(&text)
                .with_context(|| format!("invalid meta archetype JSON in '{path}'"))
        }
        _ => serde_json::from_str(EMBEDDED).context("invalid embedded meta archetype JSON"),
    }
}

/// Case-insensitive substring filters; empty/None matches everything.
pub fn filter_archetypes(
    catalog: &MetaCatalog,
    item_class: Option<&str>,
    char_class: Option<&str>,
    level_bracket: Option<&str>,
) -> Vec<MetaArchetype> {
    let matches = |needle: Option<&str>, hay: &str| match needle {
        None => true,
        Some(n) if n.trim().is_empty() => true,
        Some(n) => hay.to_lowercase().contains(&n.trim().to_lowercase()),
    };

    catalog
        .archetypes
        .iter()
        .filter(|a| matches(item_class, &a.item_class) || matches(item_class, &a.slot))
        .filter(|a| match char_class {
            None => true,
            Some(c) if c.trim().is_empty() => true,
            Some(c) => a.char_classes.iter().any(|cc| {
                cc.eq_ignore_ascii_case("any")
                    || cc.to_lowercase().contains(&c.trim().to_lowercase())
            }),
        })
        .filter(|a| {
            matches(level_bracket, &a.level_bracket) || a.level_bracket.eq_ignore_ascii_case("any")
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_catalog_loads_and_filters() -> Result<()> {
        let catalog = load_catalog()?;
        assert!(!catalog.archetypes.is_empty());
        assert_eq!(catalog.source, "curated-static");

        let bows = filter_archetypes(&catalog, Some("bow"), None, None);
        assert!(!bows.is_empty());
        assert!(bows.iter().all(|a| a.item_class.to_lowercase().contains("bow")
            || a.slot.to_lowercase().contains("bow")));

        let amazon = filter_archetypes(&catalog, Some("bow"), Some("Amazon"), Some("endgame"));
        assert!(!amazon.is_empty());

        let nothing = filter_archetypes(&catalog, Some("flask"), None, None);
        assert!(nothing.is_empty());
        Ok(())
    }
}
