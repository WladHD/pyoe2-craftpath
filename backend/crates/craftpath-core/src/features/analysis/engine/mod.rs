//! Fast calculation engine.
//!
//! The item matrix is a digraph that is *mostly* layered by
//! `target_proximity` but contains same-layer edges and genuine cycles
//! (chaos/annulment remove-and-re-add loops, intermediary temp steps), so
//! route search means **K best loopless (simple) paths** — solved exactly by
//! Yen's algorithm over a metric-monotone Dijkstra core in
//! O(K·V·(E + V log V)), replacing the legacy exponential all-simple-paths
//! DFS.
//!
//! - [`graph`]: one-time CSR snapshot of the matrix with per-edge data
//!   (chance, price) resolved exactly once.
//! - [`metrics`]: edge-decomposable metrics (chance = product, cost = sum),
//!   both monotone (extending a path never improves it) — the property
//!   Dijkstra and Yen rely on.
//! - [`yen`]: Dijkstra + Yen K-best simple paths, plus an exact
//!   branch-and-bound for the non-decomposable efficiency metric
//!   (`cost × tries60(chance)`), using `efficiency ≥ cost` to terminate.

pub mod graph;
pub mod metrics;
pub mod yen;
