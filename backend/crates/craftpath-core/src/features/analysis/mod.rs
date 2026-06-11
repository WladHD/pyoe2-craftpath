//! Route statistics: the fast engine (Yen K-best + bi-criteria search),
//! the legacy exhaustive collectors (all-path oracle + group analyzers),
//! shared helpers and the python-facing presets.

pub mod engine;
pub mod helpers;
pub mod legacy;
pub mod presets;
