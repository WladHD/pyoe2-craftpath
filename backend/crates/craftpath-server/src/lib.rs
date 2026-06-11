//! Library surface of the backend so integration tests (and potential
//! embedders) can reach the jobs layer, worker pipeline and REST router.

pub mod cli;
pub mod config;
pub mod jobs;
pub mod league;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod rest;
pub mod worker;
