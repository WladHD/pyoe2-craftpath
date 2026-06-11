//! Ergonomic façade over the calculation flow: a [`CalculationConfig`]
//! (builder) plus a [`CraftSession`] bundling providers, config and progress
//! so callers stop threading five arguments through every call.
//!
//! Purely additive — the classic `Calculator::*` functions stay untouched.

use anyhow::Result;

use crate::{
    api::{
        calculator::{
            Calculator, GroupRoute, ItemRoute, MatrixBuilder, StatisticAnalyzerCurrencyGroups,
            StatisticAnalyzerPaths,
        },
        currency::CraftCurrencyEnum,
        item::ItemSnapshot,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::THashSet,
    },
    calc::matrix::happy_path_impl::happy_path_matrix_builder_impl::HappyPathMatrixBuilder,
    progress::{NoopProgress, ProgressSink},
};

/// Calculation options with defaults matching the historical behavior.
#[derive(Clone, Debug)]
pub struct CalculationConfig {
    /// Currencies/omens excluded from matrix building (e.g. legacy omens
    /// unobtainable in the current league: Homogenising Coronation/
    /// Exaltation, Omen of Corruption). Default: none.
    pub disabled_currencies: THashSet<CraftCurrencyEnum>,
    pub max_routes: u32,
    pub max_ram_in_bytes: u64,
    /// Metadata for callers (the server layer keys league data on it).
    pub league: Option<String>,
}

impl Default for CalculationConfig {
    fn default() -> Self {
        Self {
            disabled_currencies: THashSet::default(),
            max_routes: 5,
            max_ram_in_bytes: 1_000_000_000,
            league: None,
        }
    }
}

impl CalculationConfig {
    pub fn builder() -> CalculationConfigBuilder {
        CalculationConfigBuilder {
            config: Self::default(),
        }
    }

    /// The omens that exist in the model but are unobtainable in the 0.5.0
    /// league (see MECHANICS.md) — convenience set for `disable_currencies`.
    pub fn legacy_currencies() -> [CraftCurrencyEnum; 3] {
        [
            CraftCurrencyEnum::HomogenisingCoronation(),
            CraftCurrencyEnum::HomogenisingExaltation(),
            CraftCurrencyEnum::OmenOfCorruption(),
        ]
    }
}

pub struct CalculationConfigBuilder {
    config: CalculationConfig,
}

impl CalculationConfigBuilder {
    pub fn disable_currency(mut self, currency: CraftCurrencyEnum) -> Self {
        self.config.disabled_currencies.insert(currency);
        self
    }

    pub fn disable_currencies(
        mut self,
        currencies: impl IntoIterator<Item = CraftCurrencyEnum>,
    ) -> Self {
        self.config.disabled_currencies.extend(currencies);
        self
    }

    pub fn max_routes(mut self, n: u32) -> Self {
        self.config.max_routes = n;
        self
    }

    pub fn max_ram(mut self, bytes: u64) -> Self {
        self.config.max_ram_in_bytes = bytes;
        self
    }

    pub fn league(mut self, league: impl Into<String>) -> Self {
        self.config.league = Some(league.into());
        self
    }

    pub fn build(self) -> CalculationConfig {
        self.config
    }
}

/// Bundles providers + config + progress for the whole calculation flow.
pub struct CraftSession<'a> {
    pub item_info: &'a ItemInfoProvider,
    pub market: &'a MarketPriceProvider,
    pub config: CalculationConfig,
    pub progress: &'a dyn ProgressSink,
}

impl<'a> CraftSession<'a> {
    pub fn new(item_info: &'a ItemInfoProvider, market: &'a MarketPriceProvider) -> Self {
        Self {
            item_info,
            market,
            config: CalculationConfig::default(),
            progress: &NoopProgress,
        }
    }

    pub fn with_config(mut self, config: CalculationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_progress(mut self, sink: &'a dyn ProgressSink) -> Self {
        self.progress = sink;
        self
    }

    /// Build the item matrix with the standard happy-path builder, honouring
    /// `config.disabled_currencies`.
    pub fn build_matrix(&self, start: ItemSnapshot, target: ItemSnapshot) -> Result<Calculator> {
        let builder = HappyPathMatrixBuilder::standard()
            .without_currencies(self.config.disabled_currencies.iter().cloned());
        self.build_matrix_with(start, target, &builder)
    }

    /// Build the matrix with a custom builder (the currency filter is the
    /// builder's responsibility in this variant).
    pub fn build_matrix_with(
        &self,
        start: ItemSnapshot,
        target: ItemSnapshot,
        builder: &dyn MatrixBuilder,
    ) -> Result<Calculator> {
        Calculator::generate_item_matrix_with_progress(
            start,
            target,
            self.item_info,
            self.market,
            builder,
            self.progress,
        )
    }

    pub fn analyze_paths(
        &self,
        calculator: &Calculator,
        analyzer: &dyn StatisticAnalyzerPaths,
    ) -> Result<Vec<ItemRoute>> {
        calculator.calculate_statistics_with_progress(
            self.item_info,
            self.market,
            self.config.max_routes,
            self.config.max_ram_in_bytes,
            analyzer,
            self.progress,
        )
    }

    pub fn analyze_groups(
        &self,
        calculator: &Calculator,
        analyzer: &dyn StatisticAnalyzerCurrencyGroups,
    ) -> Result<Vec<GroupRoute>> {
        calculator.calculate_statistics_currency_group_with_progress(
            self.item_info,
            self.market,
            self.config.max_ram_in_bytes,
            analyzer,
            self.progress,
        )
    }

    /// Render a route without the historical 5-argument reach-in.
    pub fn render_route(
        &self,
        calculator: &Calculator,
        route: &ItemRoute,
        analyzer: &dyn StatisticAnalyzerPaths,
        groups: Option<&Vec<GroupRoute>>,
    ) -> String {
        route.to_pretty_string(self.item_info, self.market, analyzer, calculator, groups)
    }

    pub fn render_group(
        &self,
        group: &GroupRoute,
        analyzer: &dyn StatisticAnalyzerCurrencyGroups,
    ) -> String {
        group.to_pretty_string(self.item_info, self.market, analyzer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_defaults_and_overrides() {
        let config = CalculationConfig::builder()
            .disable_currencies(CalculationConfig::legacy_currencies())
            .max_routes(7)
            .league("Standard")
            .build();

        assert_eq!(config.max_routes, 7);
        assert_eq!(config.max_ram_in_bytes, 1_000_000_000);
        assert_eq!(config.disabled_currencies.len(), 3);
        assert!(
            config
                .disabled_currencies
                .contains(&CraftCurrencyEnum::OmenOfCorruption())
        );
        assert_eq!(config.league.as_deref(), Some("Standard"));
    }
}
