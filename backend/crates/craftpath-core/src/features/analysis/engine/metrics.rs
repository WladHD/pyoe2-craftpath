//! Edge-decomposable route metrics, monomorphized into the DP hot loop.

use super::graph::GraphEdge;

pub trait EdgeMetric: Copy + 'static {
    /// `true` = smaller accumulator is better (cost); `false` = larger is
    /// better (chance).
    const LOWER_IS_BETTER: bool;

    fn init() -> f64;
    fn extend(acc: f64, edge: &GraphEdge<'_>) -> f64;
}

/// Route chance: product of edge chances (higher is better).
#[derive(Clone, Copy)]
pub struct ChanceMetric;

impl EdgeMetric for ChanceMetric {
    const LOWER_IS_BETTER: bool = false;

    #[inline]
    fn init() -> f64 {
        1.0
    }

    #[inline]
    fn extend(acc: f64, edge: &GraphEdge<'_>) -> f64 {
        acc * edge.chance_f64
    }
}

/// Route cost: sum of per-edge currency prices in Exalted (lower is better).
#[derive(Clone, Copy)]
pub struct CostMetric;

impl EdgeMetric for CostMetric {
    const LOWER_IS_BETTER: bool = true;

    #[inline]
    fn init() -> f64 {
        0.0
    }

    #[inline]
    fn extend(acc: f64, edge: &GraphEdge<'_>) -> f64 {
        acc + edge.cost_ex
    }
}

/// `true` iff `a` is strictly better than `b` under the metric.
#[inline]
pub fn better<M: EdgeMetric>(a: f64, b: f64) -> bool {
    if M::LOWER_IS_BETTER { a < b } else { a > b }
}
