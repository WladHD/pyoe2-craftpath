//! Facade: orchestration (`Calculator`), the session API, and legacy path
//! shims for the modules that moved into `domain`/`features`.

pub mod calculator;
pub mod session;

// legacy path shims
pub use crate::domain::currency;
pub use crate::domain::errors;
pub use crate::domain::item;
pub use crate::domain::provider;
pub use crate::domain::types;

pub mod calculator_utils {
    //! Moved: `craftpath_core::domain::proximity`.
    pub use crate::domain::proximity as calculate_target_proximity;
}

pub mod matrix_propagator;
