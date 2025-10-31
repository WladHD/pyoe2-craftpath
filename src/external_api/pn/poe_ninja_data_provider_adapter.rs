use anyhow::Result;

use crate::{
    api::{
        provider::market_prices::{ItemName, MarketPriceProvider, PriceInDivines},
        types::THashMap,
    },
    external_api::pn::poe_ninja_json_definition::{Data, Item, Line},
};

pub struct PoeNinjaMarketPriceProvider;

impl PoeNinjaMarketPriceProvider {
    pub fn parse_from_json(texts: &[String]) -> Result<MarketPriceProvider> {
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

        Ok(MarketPriceProvider::new(
            cache_market_prices,
            div_to_exalted,
            div_to_chaos,
        ))
    }
}

#[cfg(test)]
mod tests {
    // fn test_
}
