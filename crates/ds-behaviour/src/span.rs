//! Configured durations as the whole milliseconds a [`Stamp`] counts in.

use std::time::Duration;

use ds_core::time::stamp::Stamp;

/// `d` in whole milliseconds, saturating.
pub(crate) fn millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

/// The time `d` after `at`.
pub(crate) fn after(at: Stamp, d: Duration) -> Stamp {
    at.after(millis(d))
}
