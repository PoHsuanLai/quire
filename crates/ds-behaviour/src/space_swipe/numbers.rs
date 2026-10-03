//! design/12 §12.3.7's numbers, in integer units.

use std::time::Duration;

use super::model::{PageMilli, PagesPerSecMilli};

/// Which way a lift commits, from the distance `p` travelled since the gesture's Space and the
/// lift's velocity `v` (both positive toward later Spaces): `1`, `-1`, or `0` to return. Past
/// half a page (`|p| >= 500`) it commits in `p`'s direction unless a reverse flick
/// (`v × sign(p) <= -1500`); under half a page only on a forward flick (`v × sign(p) >= 1500`).
/// No movement commits nothing, and it never commits more than one Space.
pub fn commit(p: PageMilli, v: PagesPerSecMilli) -> i32 {
    let _ = (p, v);
    todo!("12.3.7 Commit")
}

/// How long the finish takes to cover `dp` starting at speed `v`:
/// `clamp(|dp| / max(|v|, 2 pages/s) × 3, 100 ms, 400 ms)`, so an ease-out-cubic finish starts
/// at the lift's speed.
pub fn finish_duration(dp: PageMilli, v: PagesPerSecMilli) -> Duration {
    let _ = (dp, v);
    todo!("12.3.7 Finish duration")
}

/// Where the row is drawn for a raw position `p` on a row whose Spaces run from `first` to
/// `last`: inside the row, `p` itself; past an end, `bound + 0.25 (p − bound)`, at most a quarter
/// page past it (half a page past shows an eighth).
pub fn rubber(p: PageMilli, first: u32, last: u32) -> PageMilli {
    let _ = (p, first, last);
    todo!("12.3.7 Beyond the first / last page")
}
