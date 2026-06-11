pub mod api;
pub mod domain;
pub mod features;
pub mod progress;
pub mod utils;

pub const GITHUB_REPOSITORY: &str = "WladHD/pyoe2-craftpath";

/// Flat, stable re-exports of the most-used types.
pub mod prelude {
    pub use crate::api::calculator::{
        Calculator, GroupRoute, ItemMatrix, ItemMatrixNode, ItemRoute, ItemRouteNode,
        MatrixBuilder, PropagationTarget, StatisticAnalyzerCurrencyGroups,
        StatisticAnalyzerPaths,
    };
    pub use crate::api::session::{CalculationConfig, CraftSession};
    pub use crate::domain::currency::{CraftCurrencyEnum, CraftCurrencyList};
    pub use crate::domain::fraction::Fraction;
    pub use crate::domain::item::{Item, ItemSnapshot};
    pub use crate::domain::provider::item_info::ItemInfoProvider;
    pub use crate::domain::provider::market_prices::MarketPriceProvider;
    pub use crate::domain::types::*;
    pub use crate::features::analysis::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
    pub use crate::features::analysis::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
    pub use crate::features::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
    pub use crate::progress::{NoopProgress, ProgressSink};
}

// ---------------------------------------------------------------------------
// Legacy path shims (kept >= 1 release; prefer the canonical paths above).
// ---------------------------------------------------------------------------

pub mod calc {
    //! Moved: `craftpath_core::features::{matrix, analysis}`.
    pub mod matrix {
        pub use crate::features::matrix::happy_path as happy_path_impl;
        pub use crate::features::matrix::presets;
    }
    pub mod statistics {
        pub use crate::features::analysis::engine;
        pub use crate::features::analysis::helpers;
        pub use crate::features::analysis::legacy as analyzers;
        pub use crate::features::analysis::presets;
    }
}

pub mod external_api {
    //! Moved: `craftpath_core::features::data`.
    pub use crate::features::data::coe;
    pub use crate::features::data::coe_emulator;
    pub use crate::features::data::http as fetch_json_from_urls;
    pub use crate::features::data::poe_ninja as pn;
}
