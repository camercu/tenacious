//! Observing the retry lifecycle: an `.after_attempt` hook for per-attempt
//! logging, and `.with_stats()` for a summary (`RetryStats`) after the loop ends.
//!
//! Run: `cargo run --example hooks-and-stats`
use core::time::Duration;
use relentless::clock::VirtualClock;
use relentless::{AttemptState, RetryError, RetryExt, stop, wait};

fn main() {
    // Represents a remote health check that is permanently unavailable.
    let ping_service = || -> Result<(), &str> { Err("control plane unavailable") };

    // `.with_stats()` wraps the builder so `.call()` returns (result, RetryStats).
    // Stats are useful for observability: emitting metrics or surfacing attempt counts
    // in error messages without threading a counter through the closure.
    let (result, stats) = ping_service
        .retry()
        .stop(stop::attempts(3))
        .wait(wait::fixed(Duration::from_millis(5)))
        // No `.when(...)`: the default classifier already retries every `Err`,
        // so this example stays focused on hooks and stats.
        .after_attempt(|state: &AttemptState<Result<(), &str>>| {
            if let Err(error) = state.outcome {
                eprintln!("attempt {} failed: {error}", state.attempt);
            }
        })
        .clock(VirtualClock::new())
        .with_stats()
        .call();

    assert!(matches!(result, Err(RetryError::Exhausted { .. })));
    assert_eq!(stats.attempts, 3);
}
