//! What finished runs leave behind.
//!
//! Two files in the data directory, answering two different questions. The
//! records answer "how fast have you ever been", which is one number per test
//! and only moves when you beat it; the history answers "how fast are you
//! now", which needs every run that wasn't a best as much as the ones that
//! were.
//!
//! Both are keyed by what the test *was* rather than only how long it ran —
//! see [`crate::typing::modifiers::Modifiers::key`].

use std::time::{SystemTime, UNIX_EPOCH};

pub mod history;
pub mod records;

/// Now, in unix seconds.
///
/// Shared by both files rather than kept by one and borrowed by the other, so
/// neither has to reach into the other for a clock.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}
