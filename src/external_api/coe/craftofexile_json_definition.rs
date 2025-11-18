/// This file is heavily generated with ChatGPT to create a meta-struture for serde in order to parse the json from
/// https://www.craftofexile.com/json/poe2/main/poec_data.json
/// Stuff here just needs to get parsed, since it will be transformed through an adapter anyway.
///
/// Thanks for the permission from www.craftofexile.com to use their crunched data. :)
use anyhow::Result;
use serde::Deserialize;
use serde::de::Deserializer;
use std::{collections::HashMap, collections::HashSet, fmt::Display, fs, str::FromStr};

use crate::api::types::{AffixClassEnum, AffixLocationEnum};

/// Deserialize an optional raw JSON string into HashSet<String>
fn deserialize_modgroups<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let set: HashSet<String> =
                serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
            Ok(set)
        }
        None => Ok(HashSet::new()),
    }
}

/// Deserialize:
/// HashMap<String, HashMap<String, (String|Number)>> → HashMap<u16, HashMap<u16, u16>>
fn deserialize_dir_map<'de, D>(deserializer: D) -> Result<HashMap<u16, HashMap<u16, u16>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u16),
    }

    let raw: HashMap<String, HashMap<String, StringOrNumber>> = HashMap::deserialize(deserializer)?;

    raw.into_iter()
        .map(|(outer_k, inner_map)| {
            let outer_key = outer_k.parse::<u16>().map_err(serde::de::Error::custom)?;

            let inner_map_u16 = inner_map
                .into_iter()
                .map(|(inner_k, inner_v)| {
                    let inner_key = inner_k.parse::<u16>().map_err(serde::de::Error::custom)?;
                    let inner_val = match inner_v {
                        StringOrNumber::Number(n) => n,
                        StringOrNumber::String(s) => {
                            s.parse::<u16>().map_err(serde::de::Error::custom)?
                        }
                    };
                    Ok((inner_key, inner_val))
                })
                .collect::<Result<HashMap<u16, u16>, D::Error>>()?;

            Ok((outer_key, inner_map_u16))
        })
        .collect()
}

/// Deserialize a pipe-separated string like "|20|7|13|34|" into HashSet<u8>.
/// If null or empty, returns empty set.
fn deserialize_mtypes<'de, D>(deserializer: D) -> Result<HashSet<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    let mut set = HashSet::new();

    if let Some(s) = opt {
        for part in s.split('|') {
            if part.trim().is_empty() {
                continue;
            }
            let num = part.parse::<u8>().map_err(|e| {
                serde::de::Error::custom(format!("failed to parse '{}' as u8: {}", part, e))
            })?;
            set.insert(num);
        }
    }

    Ok(set)
}

fn deserialize_mgroup_type<'de, D>(deserializer: D) -> Result<AffixClassEnum, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize either a number or a string
    struct NumberOrString;

    impl<'de> serde::de::Visitor<'de> for NumberOrString {
        type Value = AffixClassEnum;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or string representing a AffixClassEnum")
        }

        fn visit_u64<E>(self, value: u64) -> Result<AffixClassEnum, E>
        where
            E: serde::de::Error,
        {
            match value {
                1 => Ok(AffixClassEnum::Base),
                10 => Ok(AffixClassEnum::Desecrated),
                13 => Ok(AffixClassEnum::Essence),
                other => Err(E::custom(format!("unknown affix type number: {}", other))),
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<AffixClassEnum, E>
        where
            E: serde::de::Error,
        {
            // Parse string as number first
            let num: u64 = value
                .parse()
                .map_err(|_| E::custom(format!("invalid number string: {}", value)))?;
            self.visit_u64(num)
        }
    }

    deserializer.deserialize_any(NumberOrString)
}

/// Deserializes "socket", "prefix", "suffix" into AffixType.
/// Panics (errors) on unknown values.
fn deserialize_affix_type<'de, D>(deserializer: D) -> Result<AffixLocationEnum, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "socket" => Ok(AffixLocationEnum::Socket),
        "prefix" => Ok(AffixLocationEnum::Prefix),
        "suffix" => Ok(AffixLocationEnum::Suffix),
        other => Err(serde::de::Error::custom(format!(
            "unknown affix type: {}",
            other
        ))),
    }
}

/// Custom deserializer to handle both `{}` and `[]` for `ind` fields.
fn deserialize_ind<'de, D>(deserializer: D) -> Result<Option<HashMap<String, usize>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum IndHelper {
        Map(HashMap<String, usize>),
        #[allow(unused)]
        EmptyVec(Vec<serde_json::Value>),
        Null,
    }

    match IndHelper::deserialize(deserializer)? {
        IndHelper::Map(m) => Ok(Some(m)),
        IndHelper::EmptyVec(_) | IndHelper::Null => Ok(None),
    }
}

/// Deserialize HashMap<String, HashMap<String, Vec<T>>> → HashMap<u16, HashMap<u16, Vec<T>>>
fn deserialize_hashmap_u16_hashmap_u16_vec<'de, D, T>(
    deserializer: D,
) -> Result<HashMap<u16, HashMap<u16, Vec<T>>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let map: HashMap<String, HashMap<String, Vec<T>>> = HashMap::deserialize(deserializer)?;

    map.into_iter()
        .map(|(outer_k, inner_map)| {
            let outer_key = outer_k.parse::<u16>().map_err(serde::de::Error::custom)?;
            let inner_map_u16 = inner_map
                .into_iter()
                .map(|(inner_k, vec_t)| {
                    let inner_key = inner_k.parse::<u16>().map_err(serde::de::Error::custom)?;
                    Ok((inner_key, vec_t))
                })
                .collect::<Result<HashMap<u16, Vec<T>>, D::Error>>()?;
            Ok((outer_key, inner_map_u16))
        })
        .collect()
}

/// Deserialize HashMap<String, Vec<String>> → HashMap<u16, Vec<u16>>
fn deserialize_hashmap_u16_vec_u16<'de, D>(
    deserializer: D,
) -> Result<HashMap<u16, Vec<u16>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;

    map.into_iter()
        .map(|(k, v)| {
            let key = k.parse::<u16>().map_err(serde::de::Error::custom)?;
            let vec_u16 = v
                .into_iter()
                .map(|s| s.parse::<u16>().map_err(serde::de::Error::custom))
                .collect::<Result<Vec<u16>, D::Error>>()?;
            Ok((key, vec_u16))
        })
        .collect()
}

/// Allows deserializing a number or numeric string into T, returning anyhow::Result.
pub fn deserialize_from_string_or_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    T::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber<T> {
        Number(T),
        String(String),
    }

    match StringOrNumber::<T>::deserialize(deserializer)? {
        StringOrNumber::Number(n) => Ok(n),
        StringOrNumber::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
    }
}

/// Allows deserializing an Option<u16> from number, string, or null.
fn deserialize_option_u16_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Optionu16Helper {
        Number(u16),
        String(String),
        Null,
    }

    let value = Optionu16Helper::deserialize(deserializer)?;

    match value {
        Optionu16Helper::Number(n) => Ok(Some(n)),
        Optionu16Helper::String(s) => s.parse::<u16>().map(Some).map_err(serde::de::Error::custom),
        Optionu16Helper::Null => Ok(None),
    }
}

/// Parses the `data/items.json` file into a strongly-typed `GameData` struct.
pub fn parse_items_json(path: &str) -> Result<CoEGameData> {
    let content = fs::read_to_string(path)?;
    let parsed: CoEGameData = serde_json::from_str(&content)?;
    Ok(parsed)
}

/// Top-level structure of `items.json`.
#[derive(Debug, Deserialize)]
pub struct CoEGameData {
    pub bitems: Section<RawBItem>,
    pub bases: Section<RawBase>,
    pub bgroups: Section<RawBGroup>,
    pub modifiers: Section<RawModifier>,
    pub mgroups: Section<RawMGroup>,
    pub mtypes: Section<RawMType>,
    pub fossils: Section<RawFossil>,
    pub catalysts: Section<RawCatalyst>,
    pub essences: Section<RawEssence>,

    #[serde(deserialize_with = "deserialize_hashmap_u16_hashmap_u16_vec")]
    pub tiers: HashMap<u16, HashMap<u16, Vec<RawTierEntry>>>,
    #[serde(deserialize_with = "deserialize_hashmap_u16_vec_u16")]
    pub basemods: HashMap<u16, Vec<u16>>,
    #[serde(deserialize_with = "deserialize_hashmap_u16_vec_u16")]
    pub modbases: HashMap<u16, Vec<u16>>,

    pub clngs: Vec<RawLanguage>,

    pub socketables: RawSocketablesSection,

    #[serde(default)]
    pub dir: Option<HashMap<String, HashMap<String, String>>>,

    #[serde(default)]
    pub name: Option<HashMap<String, usize>>,
}

/// Generic section of the JSON containing a sequence and optional index map.
#[derive(Debug, Deserialize)]
pub struct Section<T> {
    pub seq: Vec<T>,

    #[serde(default, deserialize_with = "deserialize_ind")]
    pub ind: Option<HashMap<String, usize>>,

    /// Some sections (e.g., bases) also contain an "items" map.
    #[serde(default)]
    pub items: Option<HashMap<String, Vec<usize>>>,

    #[serde(default, deserialize_with = "deserialize_dir_map")]
    pub dir: HashMap<u16, HashMap<u16, u16>>,
}

// -------------------- Entities --------------------

/// "bitems" entries — individual base items.
#[derive(Debug, Deserialize)]
pub struct RawBItem {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_bitem: u16,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_base: u16,
    pub name_bitem: String,
    pub drop_level: String,
    pub properties: Option<String>,
    pub requirements: Option<String>,
    pub implicits: Option<String>,
    pub exp: String,
    pub imgurl: Option<String>,
    pub is_legacy: String,
    pub exmods: Option<String>,
}

/// "bases" entries — base types like "Amulet".
#[derive(Debug, Deserialize)]
pub struct RawBase {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_bgroup: u16,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_base: u16,
    pub name_base: String,
    pub is_jewellery: String,
    pub base_type: String,
    pub has_childs: String,
    #[serde(deserialize_with = "deserialize_option_u16_from_string_or_number")]
    pub master_base: Option<u16>,
    pub unique_notable: String,
    pub enchant: Option<String>,
    pub is_legacy: String,
    pub is_martial: String,
}

/// "bgroups" entries — base groups like "Boots".
#[derive(Debug, Deserialize)]
pub struct RawBGroup {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_bgroup: u16,
    pub name_bgroup: String,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub max_affix: u8,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_rare: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_influenced: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_fossil: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_ess: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_craftable: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_notable: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub is_catalyst: bool,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub has_items: bool,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub max_sockets: u8,
}

/// "modifiers" entries — affixes.
#[derive(Debug, Deserialize, Clone)]
pub struct RawModifier {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_modifier: u16,
    pub modgroup: Option<String>,
    #[serde(deserialize_with = "deserialize_modgroups")]
    pub modgroups: HashSet<String>,
    #[serde(deserialize_with = "deserialize_affix_type")]
    pub affix: AffixLocationEnum,
    #[serde(deserialize_with = "deserialize_mgroup_type")]
    pub id_mgroup: AffixClassEnum,
    pub name_modifier: String,
    pub id_fossil: Option<String>,
    #[serde(deserialize_with = "deserialize_mtypes")]
    pub mtypes: HashSet<u8>,
    pub meta: Option<String>,
    pub mtags: Option<String>,
    pub hybrid: String,
    pub notable: String,
    pub vex: String,
    pub amg: Option<String>,
    pub exkey: Option<String>,
    pub ubt: Option<String>,
    pub tgb: Option<String>,
    pub ntgb: Option<String>,
    pub hr: bool,
    pub ha: bool,
}

/// "mgroups" entries — modifier groups.
#[derive(Debug, Deserialize)]
pub struct RawMGroup {
    pub is_influence: String,
    pub id_mgroup: String,
    pub name_mgroup: String,
    pub poedb_id: Option<String>,
    pub paste_link: Option<String>,
    pub is_main: String,
    pub max_chosen: String,
    pub is_compute: String,
}

/// "mtypes" entries — modifier type definitions.
#[derive(Debug, Deserialize)]
pub struct RawMType {
    pub id_mtype: String,
    pub poedb_id: String,
    pub jewellery_tag: String,
    pub harvest: String,
    pub tangled: String,
    pub parent_id: Option<String>,
    pub name_mtype: String,
}

/// "fossils" entries — empty in the sample.
#[derive(Debug, Deserialize)]
pub struct RawFossil {}

/// "catalysts" entries.
#[derive(Debug, Deserialize)]
pub struct RawCatalyst {
    pub id_catalyst: String,
    pub name_catalyst: String,
    pub tags: String,
}

/// "essences" entries.
#[derive(Debug, Deserialize)]
pub struct RawEssence {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_essence: u16,
    pub name_essence: String,
    pub tooltip: String,
    #[serde(deserialize_with = "deserialize_tiers")]
    pub tiers: HashMap<u16, Vec<Vec<RawTierMod>>>,
    #[serde(deserialize_with = "deserialize_bool_from_01")]
    pub corrupt: bool,
}

fn deserialize_bool_from_01<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(serde::de::Error::custom(format!(
            "invalid bool value: {}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize)]
pub struct RawTierMod {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub r#mod: u16,
    pub id: String,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub ilvl: u8,
}

fn deserialize_tiers<'de, D>(
    deserializer: D,
) -> Result<HashMap<u16, Vec<Vec<RawTierMod>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

/// "tiers" entries (nested inside tiers map).
#[derive(Debug, Deserialize)]
pub struct RawTierEntry {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub ilvl: u8,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub weighting: u32,
    pub nvalues: Option<String>,
    pub tord: i32,
    pub alias: Option<String>,
}

/// "clngs" entries — language info.
#[derive(Debug, Deserialize)]
pub struct RawLanguage {
    pub name: String,
    pub code: String,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id: u16,
    pub main: bool,
}

// -------------------- Socketables --------------------

/// "socketables" section — has seq, ind, plus bytype/bybase maps.
#[derive(Debug, Deserialize)]
pub struct RawSocketablesSection {
    pub seq: Vec<Socketable>,
    #[serde(default, deserialize_with = "deserialize_ind")]
    pub ind: Option<HashMap<String, usize>>,

    /// Maps socketable type (e.g. "rune", "talisman") to lists of IDs.
    #[serde(default)]
    pub bytype: Option<HashMap<String, Vec<String>>>,

    /// Maps base type (e.g. "armour", "weapons") to lists of IDs.
    #[serde(default)]
    pub bybase: Option<HashMap<String, Vec<String>>>,
}

/// Individual socketable entries.
#[derive(Debug, Deserialize)]
pub struct Socketable {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id_socketable: u16,
    pub stype: String,
    pub name_socketable: String,
    pub mods: String,
    pub imgurl: String,
}
