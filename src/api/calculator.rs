use std::hash::{Hash, Hasher};

use anyhow::Result;
use tracing::instrument;

use crate::{
    api::{
        currency::CraftCurrencyList,
        item::{Item, ItemSnapshot},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashMap,
    },
    utils::fraction_utils::Fraction,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct PropagationTarget {
    pub next: ItemSnapshot,
    pub chance: Fraction,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct ItemMatrixNode {
    pub item: Item,
    pub propagate: THashMap<CraftCurrencyList, Vec<PropagationTarget>>,
}

crate::derive_DebugDisplay!(PropagationTarget, ItemMatrixNode, Calculator);

pub type ItemMatrix = THashMap<u64, ItemMatrixNode>;

// do not include references ??
// item and chance are w/e since sizewise nothing changes u64 + u32 + u32 (+ struct)
// but HashSet could be costly?? if to much revert to ref
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, get_all, frozen, hash, str)
)]
pub struct ItemRouteNode {
    pub item: u64,
    pub chance: Fraction,
    pub currency_list: CraftCurrencyList,
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(ItemRouteNode, ItemRoute, StatisticResult);

// this needs to be converted to Python types either way
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, get_all, frozen, hash, str)
)]
pub struct ItemRoute {
    pub route: Vec<ItemRouteNode>,
    pub weight: f64, // for internal 15-17 digit precision, i think inaccuracies on deep paths are acceptable, if not swap to rust_decimal
}

impl PartialEq for ItemRoute {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route
    }
}

impl Eq for ItemRoute {}

impl Hash for ItemRoute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.route.hash(state);
    }
}

pub trait MatrixBuilder {
    fn get_name() -> &'static str;
    fn get_description() -> &'static str;
    fn generate_item_matrix(
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_info: &ItemInfoProvider,
    ) -> Result<ItemMatrix>;
}

pub trait StatisticProvider {
    fn get_name() -> &'static str;
    fn get_description() -> &'static str;
    fn get_unit_type() -> &'static str;
    fn lower_is_better() -> bool;
    fn get_statistic<'a>(
        matrix: &'a ItemMatrix,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<ItemRoute>>;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(weakref, from_py_object, get_all, frozen, str)
)]
pub struct StatisticResult {
    pub sorted_routes: Vec<ItemRoute>,
    pub lower_is_better: bool,
    pub unit_type: &'static str,
}

#[derive(Debug)]
pub struct Calculator;

impl Calculator {
    #[instrument(skip_all)]
    fn generate_item_matrix<B: MatrixBuilder>(
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_provider: &ItemInfoProvider,
    ) -> Result<ItemMatrix> {
        tracing::info!("Using '{}' to generate item matrix ...", B::get_name());
        tracing::info!("Description: {}", B::get_description());
        let res = B::generate_item_matrix(starting_item, target, item_provider)?;
        tracing::info!("Successfully generated item matrix. (TODO SHOW STATS)");

        Ok(res)
    }

    #[instrument(skip_all)]
    fn calculate_statistics<S: StatisticProvider>(
        matrix: &ItemMatrix,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
    ) -> Result<StatisticResult> {
        tracing::info!("Using '{}' to calculate statistics ...", S::get_name());
        tracing::info!("Description: {}", S::get_description());
        let res = S::get_statistic(matrix, item_provider, market_provider, max_ram_in_bytes)?;
        tracing::info!("Successfully calculated statistics. (TODO SHOW STATS)");

        Ok(StatisticResult {
            sorted_routes: res,
            lower_is_better: S::lower_is_better(),
            unit_type: S::get_unit_type(),
        })
    }

    #[instrument(skip_all)]
    fn calculate_target_proximity(
        start: &ItemSnapshot,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> u8 {
        todo!()

        // return 0 if target item AFFIXES reached -> can be followed with some socketing shenanigans or sth
        // return 12 on max distance
    }

    #[instrument(skip_all)]
    fn sanity_check_item(start: &ItemSnapshot, provider: &ItemInfoProvider) -> bool {
        todo!()

        // provide an item and check if the selected mods are reachable.
        // e. g. exclusive mods, multiple fractures etc.
    }
}
