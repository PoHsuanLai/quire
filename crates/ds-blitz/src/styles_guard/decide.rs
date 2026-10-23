//! The guard's decision, pure: from the records of earlier starts and the sheets about to load,
//! load them or not (customization design section 7.4).

use super::record::{Reached, Record};
use ds::style::kit::Fingerprint;

/// How many starts in a row, with the same sheets, must fail before the guard steps in.
pub const FAILED_STARTS: usize = 2;

/// Whether the program loads the look and the person's sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Load {
    /// Load them.
    WithStyles,
    /// Start without them and say so: the last starts with these sheets never got far enough.
    Without,
}

/// `Without` when each of the last [`FAILED_STARTS`] records is a start with `now` that never
/// reached `Good` (two crashes, or no first frame within ten seconds); otherwise `WithStyles`.
/// A record of other sheets, or a `Good` one, breaks the run: a changed sheet gets a fresh try.
pub fn decide(records: &[Record], now: Fingerprint) -> Load {
    let tail = records.len().saturating_sub(FAILED_STARTS);
    let last = &records[tail..];
    let failed = last.len() == FAILED_STARTS
        && last
            .iter()
            .all(|record| record.hash == now && record.reached == Reached::Starting);
    if failed {
        Load::Without
    } else {
        Load::WithStyles
    }
}
