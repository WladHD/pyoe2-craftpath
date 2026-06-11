use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_from_string_or_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
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

fn bool_from_any<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match val {
        None => false, // default if missing
        Some(serde_json::Value::Bool(b)) => b,
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) != 0,
        Some(serde_json::Value::String(s)) => s == "1",
        _ => panic!("Expected bool value but found '{:?}'", val),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmulatorItemExport {
    pub settings: Settings,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub bgroup: u16,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub base: u16, // mark
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub bitem: u16, // mark
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub ilvl: u8, // mark
    pub rarity: String, // mark <-- convert
    // pub influences: Option<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub sockets: u8,
    // pub socketed: Vec<serde_json::Value>, // TODO not yet ...
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub quality: u8,
    // pub exmods: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "bool_from_any")]
    pub corrupted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub implicits: Option<serde_json::Value>,
    pub iaffixes: Vec<IAffix>, // only this needed for now
                               // pub iaffbt: IAFFBT,
                               // pub imprint: Option<serde_json::Value>,
                               // pub eldritch: Option<serde_json::Value>,
                               // pub meta_flags: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IAffix {
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub id: u16, // only this needed for now
    #[serde(default, deserialize_with = "bool_from_any")]
    pub frac: bool, // maybe this
    //// transitive over id resolvable
    // pub mgrp: String, // transitive over id
    // pub atype: String,
    // pub modgroups: Vec<String>,
    /// conversion = length of tiers - tindex
    #[serde(deserialize_with = "deserialize_from_string_or_number")]
    pub tindex: u8,
    // pub bench: u32,
    // pub maven: u32,
    // pub nvalues: String,
    // pub rolls: Vec<u32>,
    // pub weight: String,
}
