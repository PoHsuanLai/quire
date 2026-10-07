//! Wall time as plain data, so the model stays pure and a test names the instant.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Seconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Epoch(pub i64);

/// How long an item stays in Today after it was last opened.
pub const IDLE: Duration = Duration::from_secs(12 * 60 * 60);

impl Epoch {
    /// The wall clock now.
    pub fn now() -> Epoch {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        Epoch(i64::try_from(secs).unwrap_or(i64::MAX))
    }

    /// `self` moved later by `by`.
    pub fn plus(self, by: Duration) -> Epoch {
        Epoch(
            self.0
                .saturating_add(i64::try_from(by.as_secs()).unwrap_or(i64::MAX)),
        )
    }

    /// How long after `earlier` this is, or `None` when it is not after it.
    pub fn since(self, earlier: Epoch) -> Option<Duration> {
        u64::try_from(self.0.saturating_sub(earlier.0))
            .ok()
            .map(Duration::from_secs)
    }
}
