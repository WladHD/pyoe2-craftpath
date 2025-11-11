use serde::{Deserialize, Serialize};

use crate::{
    api::{currency::CraftCurrencyEnum, provider::item_info::ItemInfoProvider, types::THashMap},
    explicit_type,
};

explicit_type!(ItemName, String);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, str, get_all))]
pub struct MarketPriceProvider {
    pub cache_market_prices: THashMap<ItemName, PriceInDivines>,
    pub cache_exchange_rate_div_to_exalted: f64,
    pub cache_exchange_rate_div_to_chaos: f64,
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl MarketPriceProvider {
    pub fn try_lookup_currency_in_divines_default_if_fail(
        &self,
        currency: &CraftCurrencyEnum,
        item_provider: &ItemInfoProvider,
    ) -> PriceInDivines {
        let item_name = ItemName::from(currency.get_item_name(&item_provider).to_string());
        let res = self
            .cache_market_prices
            .get(&item_name)
            .cloned()
            .unwrap_or_else(|| {
                // tracing::warn!("Could not find price in divines for '{}' ... using 1 Exalted Orb as placeholder value.", item_name.get_raw_value());
                PriceInDivines {
                    raw_value: self.currency_convert_from_exalted(1_f64, &PriceKind::Divine),
                }
            });

        res
    }
    pub fn currency_convert(&self, from_divs: &PriceInDivines, to_kind: &PriceKind) -> f64 {
        let to_ex = self.cache_exchange_rate_div_to_exalted;
        let to_chaos = self.cache_exchange_rate_div_to_chaos;

        match to_kind {
            PriceKind::Divine => from_divs.get_divine_value(),
            PriceKind::Exalted => from_divs.get_divine_value() * to_ex,
            PriceKind::Chaos => from_divs.get_divine_value() * to_chaos,
        }
    }

    pub fn currency_convert_raw(&self, from_divs: f64, to_kind: &PriceKind) -> f64 {
        let to_ex = self.cache_exchange_rate_div_to_exalted;
        let to_chaos = self.cache_exchange_rate_div_to_chaos;

        match to_kind {
            PriceKind::Divine => from_divs,
            PriceKind::Exalted => from_divs * to_ex,
            PriceKind::Chaos => from_divs * to_chaos,
        }
    }

    pub fn currency_convert_from_exalted(&self, from_ex: f64, to_kind: &PriceKind) -> f64 {
        let div_to_ex = self.cache_exchange_rate_div_to_exalted;
        let div_to_chaos = self.cache_exchange_rate_div_to_chaos;

        match to_kind {
            PriceKind::Divine => from_ex / div_to_ex, // Exalted → Divine
            PriceKind::Exalted => from_ex,            // Exalted → Exalted
            PriceKind::Chaos => from_ex * (div_to_chaos / div_to_ex), // Exalted → Chaos
        }
    }
}

// todo generate conversion etc
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(ord, eq, weakref, from_py_object, str))]
pub struct PriceInDivines {
    raw_value: f64,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub enum PriceKind {
    Divine,
    Exalted,
    Chaos,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl PriceInDivines {
    #[new]
    pub fn new(value: f64) -> Self {
        Self { raw_value: value }
    }

    pub fn get_divine_value(&self) -> f64 {
        return self.raw_value;
    }
}

#[cfg(not(feature = "python"))]
impl PriceInDivines {
    pub fn new(value: f64) -> Self {
        Self { raw_value: value }
    }

    pub fn get_divine_value(&self) -> f64 {
        return self.raw_value;
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(PriceInDivines, MarketPriceProvider);
