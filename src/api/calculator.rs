use std::hash::{Hash, Hasher};

use crate::api::calculator_utils::calculate_target_proximity::calculate_target_proximity;
use crate::api::errors::CraftPathError;
use crate::api::item::ItemTechnicalMeta;
use crate::calc::statistics::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
use crate::calc::statistics::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
use crate::{
    api::{
        currency::CraftCurrencyList,
        item::{Item, ItemSnapshot},
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashMap,
    },
    calc::matrix::matrix_builder_presets::MatrixBuilderPreset,
    utils::fraction_utils::Fraction,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct PropagationTarget {
    pub next: ItemSnapshot,
    pub chance: Fraction,
    pub meta: ItemTechnicalMeta,
}

impl PropagationTarget {
    pub fn new(chance: Fraction, next: ItemSnapshot) -> Self {
        Self {
            next,
            chance,
            meta: ItemTechnicalMeta::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct ItemMatrixNode {
    pub item: Item,
    pub propagate: THashMap<CraftCurrencyList, Vec<PropagationTarget>>,
}

pub type ItemMatrix = THashMap<u64, ItemMatrixNode>;

// do not include references ??
// item and chance are w/e since sizewise nothing changes u64 + u32 + u32 (+ struct)
// but HashSet could be costly?? if to much revert to ref
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, get_all, frozen, hash, str)
)]
pub struct ItemRouteNode {
    pub item_matrix_id: u64,
    pub chance: Fraction,
    pub currency_list: CraftCurrencyList,
}

// this needs to be converted to Python types either way
#[derive(Clone, Debug, Serialize, Deserialize)]
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

pub trait MatrixBuilder: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn generate_item_matrix(
        &self,
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_info: &ItemInfoProvider,
        market_info: &MarketPriceProvider,
    ) -> Result<ItemMatrix>;
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(str))]
pub struct DynMatrixBuilder(pub Box<dyn MatrixBuilder + Send + Sync>);

impl std::fmt::Display for DynMatrixBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Matrix Builder ({})\nDescription: {}",
            self.0.get_name(),
            self.0.get_description()
        )
    }
}

pub trait StatisticAnalyzerPaths {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn get_unit_type(&self) -> &'static str;
    fn lower_is_better(&self) -> bool;
    fn get_statistic(
        &self,
        calculator: &Calculator,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_routes: u32,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<ItemRoute>>;
}

pub trait StatisticAnalyzerCurrencyGroups {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn get_unit_type(&self) -> &'static str;
    fn lower_is_better(&self) -> bool;
    fn get_statistic(
        &self,
        calculator: &Calculator,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
    ) -> Result<Vec<(Vec<CraftCurrencyList>, f64, Vec<Vec<f64>>)>>;
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(str))]
pub struct DynStatisticAnalyzerPaths(pub Box<dyn StatisticAnalyzerPaths + Send + Sync>);

impl std::fmt::Display for DynStatisticAnalyzerPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistic Analyzer ({})\nDescription: {}\nLower is better? {}",
            self.0.get_name(),
            self.0.get_description(),
            self.0.lower_is_better(),
        )
    }
}

#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(str))]
pub struct DynStatisticAnalyzerCurrencyGroups(
    pub Box<dyn StatisticAnalyzerCurrencyGroups + Send + Sync>,
);

impl std::fmt::Display for DynStatisticAnalyzerCurrencyGroups {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistic Analyzer ({})\nDescription: {}",
            self.0.get_name(),
            self.0.get_description()
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(weakref, from_py_object, get_all, frozen, str)
)]
pub struct StatisticResult {
    pub sorted_routes: Vec<ItemRoute>,
    pub lower_is_better: bool,
    pub unit_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(weakref, from_py_object, get_all, str))]
pub struct Calculator {
    pub matrix: ItemMatrix,
    pub starting_item: ItemSnapshot,
    pub target_item: ItemSnapshot,
    pub statistics: THashMap<String, StatisticResult>,
}

impl Calculator {
    #[instrument(skip_all)]
    pub fn generate_item_matrix(
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_provider: &ItemInfoProvider,
        market_info: &MarketPriceProvider,
        matrix_builder: MatrixBuilderPreset,
    ) -> Result<Self> {
        let matrix_builder = matrix_builder.get_matrix_builder_instance();

        tracing::info!(
            "Using '{}' to generate item matrix ...",
            matrix_builder.0.get_name()
        );
        tracing::info!("Description: {}", matrix_builder.0.get_description());

        let res = matrix_builder.0.generate_item_matrix(
            starting_item.clone(),
            target.clone(),
            item_provider,
            market_info,
        )?;

        let reached = res
            .iter()
            .any(|test| test.1.item.helper.target_proximity == 0);

        if !reached {
            return Err(CraftPathError::ItemMatrixCouldNotReachTarget().into());
        }

        tracing::info!("Successfully generated item matrix. (TODO SHOW STATS)");

        Ok(Self {
            matrix: res,
            starting_item: starting_item,
            target_item: target,
            statistics: THashMap::default(),
        })
    }

    #[instrument(skip_all)]
    pub fn calculate_statistics(
        &mut self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_routes: u32,
        max_ram_in_bytes: u64,
        statistic_analyzer: StatisticAnalyzerPathPreset,
    ) -> Result<&StatisticResult> {
        let statistic_analyzer = statistic_analyzer.get_statistic_analyzer_instance();
        tracing::info!(
            "Using '{}' to calculate statistics ...",
            statistic_analyzer.0.get_name()
        );
        tracing::info!("Description: {}", statistic_analyzer.0.get_description());
        let res = statistic_analyzer.0.get_statistic(
            &self,
            item_provider,
            market_provider,
            max_routes,
            max_ram_in_bytes,
        )?;
        tracing::info!("Successfully calculated statistics. (TODO SHOW STATS)");

        self.statistics.insert(
            statistic_analyzer.0.get_name().to_string(),
            StatisticResult {
                sorted_routes: res,
                lower_is_better: statistic_analyzer.0.lower_is_better(),
                unit_type: statistic_analyzer.0.get_unit_type().to_string(),
            },
        );

        Ok(self
            .statistics
            .get(statistic_analyzer.0.get_name())
            .unwrap())
    }

    #[instrument(skip_all)]
    pub fn calculate_statistics_currency_group(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
        statistic_analyzer: StatisticAnalyzerCurrencyGroupPreset,
    ) -> Result<Vec<(Vec<CraftCurrencyList>, f64, Vec<Vec<f64>>)>> {
        let statistic_analyzer = statistic_analyzer.get_statistic_analyzer_instance();
        tracing::info!(
            "Using '{}' to calculate statistics ...",
            statistic_analyzer.0.get_name()
        );
        tracing::info!("Description: {}", statistic_analyzer.0.get_description());
        let res = statistic_analyzer.0.get_statistic(
            &self,
            item_provider,
            market_provider,
            max_ram_in_bytes,
        )?;
        tracing::info!("Successfully calculated statistics. (TODO SHOW STATS)");

        Ok(res)
    }

    #[instrument(skip_all)]
    pub fn calculate_target_proximity(
        start: &ItemSnapshot,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<u8> {
        // return 0 if target item AFFIXES reached -> can be followed with some socketing shenanigans or sth
        // return 12 on max distance
        calculate_target_proximity(start, target, provider)
    }

    #[instrument(skip_all)]
    pub fn sanity_check_item(_start: &ItemSnapshot, _provider: &ItemInfoProvider) -> bool {
        todo!()

        // provide an item and check if the selected mods are reachable.
        // e. g. exclusive mods, multiple fractures etc.
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl Calculator {
    #[staticmethod]
    #[pyo3(name = "generate_item_matrix")]
    fn generate_item_matrix_py(
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_provider: &ItemInfoProvider,
        market_info: &MarketPriceProvider,
        matrix_builder: MatrixBuilderPreset,
    ) -> pyo3::PyResult<Self> {
        Calculator::generate_item_matrix(
            starting_item,
            target,
            item_provider,
            market_info,
            matrix_builder,
        )
        .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[pyo3(name = "calculate_statistics")]
    fn calculate_statistics_py(
        &mut self,
        py: pyo3::Python,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_routes: u32,
        max_ram_in_bytes: u64,
        statistic_analyzer: StatisticAnalyzerPathPreset,
    ) -> pyo3::PyResult<StatisticResult> {
        // allow parallelization
        py.detach(move || {
            self.calculate_statistics(
                item_provider,
                market_provider,
                max_routes,
                max_ram_in_bytes,
                statistic_analyzer,
            )
            .cloned()
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
        })
    }

    #[pyo3(name = "calculate_statistics_currency_group")]
    pub fn calculate_statistics_currency_group_py(
        &self,
        item_provider: &ItemInfoProvider,
        market_provider: &MarketPriceProvider,
        max_ram_in_bytes: u64,
        statistic_analyzer: StatisticAnalyzerCurrencyGroupPreset,
    ) -> pyo3::PyResult<Vec<(Vec<CraftCurrencyList>, f64, Vec<Vec<f64>>)>> {
        self.calculate_statistics_currency_group(
            item_provider,
            market_provider,
            max_ram_in_bytes,
            statistic_analyzer,
        )
        .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "calculate_target_proximity")]
    fn calculate_target_proximity_py(
        start: &ItemSnapshot,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> pyo3::PyResult<u8> {
        Calculator::calculate_target_proximity(start, target, provider)
            .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "sanity_check_item")]
    fn sanity_check_item_py(start: &ItemSnapshot, provider: &ItemInfoProvider) -> bool {
        Calculator::sanity_check_item(start, provider)
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(
    PropagationTarget,
    ItemMatrixNode,
    Calculator,
    ItemRouteNode,
    ItemRoute,
    StatisticResult
);
