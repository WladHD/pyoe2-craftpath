use crate::{api::types::THashMap, derive_DebugDisplay, explicit_type};

explicit_type!(ItemName, String);

#[cfg(not(feature = "python"))]
pub struct MarketPriceProvider {
    cache_market_prices: THashMap<ItemName, PriceInDivines>,
    cache_exchange_rate_div_to_exalted: PriceInDivines,
    cache_exchange_rate_div_to_chaos: PriceInDivines,
}

#[cfg(feature = "python")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, str))]
pub struct MarketPriceProvider {
    #[pyo3(get)]
    cache_market_prices: THashMap<ItemName, PriceInDivines>,
    #[pyo3(get)]
    cache_exchange_rate_div_to_exalted: f64,
    #[pyo3(get)]
    cache_exchange_rate_div_to_chaos: f64,
}

impl MarketPriceProvider {
    pub fn new(
        cache_market_prices: THashMap<ItemName, PriceInDivines>,
        cache_exchange_rate_div_to_exalted: f64,
        cache_exchange_rate_div_to_chaos: f64,
    ) -> Self {
        Self {
            cache_market_prices,
            cache_exchange_rate_div_to_exalted,
            cache_exchange_rate_div_to_chaos,
        }
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl MarketPriceProvider {
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
}

// todo generate conversion etc
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(ord, eq, weakref, from_py_object, str))]
pub struct PriceInDivines {
    raw_value: f64,
}

#[cfg(not(feature = "python"))]
impl PriceInDivines {
    pub fn new(value: f64) -> Self {
        Self { raw_value: value }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
derive_DebugDisplay!(PriceInDivines, MarketPriceProvider);
