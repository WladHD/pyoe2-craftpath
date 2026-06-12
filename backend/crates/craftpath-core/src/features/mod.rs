//! Feature modules (bulletproof-style): each feature owns its logic and
//! tests, imports only `domain` + `utils` + `progress`, and never reaches
//! into a sibling feature.

pub mod analysis;
pub mod craftspec;
pub mod data;
pub mod inspect;
pub mod matrix;
pub mod render;
