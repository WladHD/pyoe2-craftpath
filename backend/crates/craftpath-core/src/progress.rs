//! Minimal progress/cancellation hook threaded through the long-running
//! calculation entry points.
//!
//! The calculation hot loops only call [`ProgressSink::report`] /
//! [`ProgressSink::is_cancelled`] every couple hundred thousand iterations,
//! so implementations may do cheap work (atomics, formatting) but should not
//! block (no I/O) — push I/O into a sampler thread instead.

/// Receiver for progress updates from long-running calculations.
pub trait ProgressSink: Send + Sync {
    /// `message` is a human-readable status line (same text the CLI spinner
    /// used to display), `current` a monotone counter for the running phase,
    /// `total` the phase total if known (usually unknown — route counts are
    /// only known once collection finishes).
    fn report(&self, message: &str, current: u64, total: Option<u64>);

    /// Polled at the same cadence as `report`. Returning `true` makes the
    /// calculation abort with [`CraftPathError::Cancelled`]
    /// (crate::api::errors::CraftPathError::Cancelled).
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Default sink: ignores progress, never cancels.
pub struct NoopProgress;

impl ProgressSink for NoopProgress {
    fn report(&self, _message: &str, _current: u64, _total: Option<u64>) {}
}
