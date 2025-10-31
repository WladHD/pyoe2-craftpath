// item struct to parse poe.ninja's api
// e. g. https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Rise%20of%20the%20Abyssal&type=Currency
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Data {
    pub core: Core,
    pub lines: Vec<Line>,
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Core {
    pub items: Vec<Item>,
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub exalted: f64,
    pub chaos: f64,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Line {
    pub id: String,
    #[serde(rename = "primaryValue")]
    pub primary_value: f64,
}
