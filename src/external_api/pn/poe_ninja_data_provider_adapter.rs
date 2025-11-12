use anyhow::Result;

use crate::{
    api::{
        provider::market_prices::{ItemName, MarketPriceProvider, PriceInDivines},
        types::THashMap,
    },
    external_api::pn::poe_ninja_json_definition::{Data, Item, Line},
};

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub struct PoeNinjaMarketPriceProvider;

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::prelude::pymethods)]
impl PoeNinjaMarketPriceProvider {
    #[pyo3(name = "parse_from_json_list")]
    #[staticmethod]
    pub fn parse_from_json_list_py(texts: Vec<String>) -> pyo3::PyResult<MarketPriceProvider> {
        Self::parse_from_json_list(texts.as_ref())
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }
}

impl PoeNinjaMarketPriceProvider {
    pub fn parse_from_json_list(texts: &[String]) -> Result<MarketPriceProvider> {
        let mut combined_lines: Vec<Line> = Vec::new();
        let mut combined_items: THashMap<String, Item> = THashMap::default();

        let mut div_to_exalted = 0.0;
        let mut div_to_chaos = 0.0;

        for text in texts {
            let data: Data = serde_json::from_str(text)?;

            // cache exchange rates (take from the first JSON)
            if div_to_exalted == 0.0 {
                div_to_exalted = data.core.rates.exalted;
            }
            if div_to_chaos == 0.0 {
                div_to_chaos = data.core.rates.chaos;
            }

            // append all lines
            combined_lines.extend(data.lines);

            // append all items
            for item in data.items {
                combined_items.insert(item.id.clone(), item);
            }
        }

        // Build cache_market_prices
        let mut cache_market_prices: THashMap<ItemName, PriceInDivines> = THashMap::default();

        for line in combined_lines {
            if let Some(item) = combined_items.get(&line.id) {
                cache_market_prices.insert(
                    ItemName::from(item.name.clone()),
                    PriceInDivines::new(line.primary_value),
                );
            } else {
                panic!("COULD NOT FIND {:?}", line);
            }
        }

        Ok(MarketPriceProvider {
            cache_exchange_rate_div_to_chaos: div_to_chaos,
            cache_exchange_rate_div_to_exalted: div_to_exalted,
            cache_market_prices,
        })
    }
}

#[cfg(test)]
mod tests {
    // fn test_
}
