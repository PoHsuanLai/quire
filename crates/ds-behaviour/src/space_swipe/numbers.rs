//! design/12 §12.3.7's numbers, in integer units.

use std::time::Duration;

use super::model::{PageMilli, PagesPerSecMilli};

/// Half a page, where a drag alone commits.
const HALF_PAGE: i32 = 500;

/// The speed, in thousandths of a page per second, that decides a commit on its own.
const FLICK: i32 = 1500;

/// The slowest speed a finish is timed from, so a slow lift still arrives.
const SLOWEST: u64 = 2000;

/// The shortest and longest finish, in milliseconds.
const FINISH_MS: (u64, u64) = (100, 400);

/// The farthest, in thousandths of a page, the row shows past an end.
const BAND_CAP: i32 = 250;

/// Which way a lift commits, from the distance `p` travelled since the gesture's Space and the
/// lift's velocity `v` (both positive toward later Spaces): `1`, `-1`, or `0` to return. Past
/// half a page (`|p| >= 500`) it commits in `p`'s direction unless a reverse flick
/// (`v × sign(p) <= -1500`); under half a page only on a forward flick (`v × sign(p) >= 1500`).
/// No movement commits nothing, and it never commits more than one Space.
pub fn commit(p: PageMilli, v: PagesPerSecMilli) -> i32 {
    let direction = p.0.signum();
    let toward = v.0.saturating_mul(direction);
    let flicked_back = toward <= -FLICK;
    let flicked_on = toward >= FLICK;
    match p.0.unsigned_abs() >= HALF_PAGE.unsigned_abs() {
        true if flicked_back => 0,
        true => direction,
        false if flicked_on => direction,
        false => 0,
    }
}

/// How long the finish takes to cover `dp` starting at speed `v`:
/// `clamp(|dp| / max(|v|, 2 pages/s) × 3, 100 ms, 400 ms)`, so an ease-out-cubic finish starts
/// at the lift's speed. Whole milliseconds.
pub fn finish_duration(dp: PageMilli, v: PagesPerSecMilli) -> Duration {
    let speed = u64::from(v.0.unsigned_abs()).max(SLOWEST);
    let ms = u64::from(dp.0.unsigned_abs()) * 3000 / speed;
    Duration::from_millis(ms.clamp(FINISH_MS.0, FINISH_MS.1))
}

/// Where the row is drawn for a raw position `p` on a row whose Spaces run from `first` to
/// `last`: inside the row, `p` itself; past an end, `bound + 0.25 (p − bound)`, at most a quarter
/// page past it (half a page past shows an eighth).
pub fn rubber(p: PageMilli, first: u32, last: u32) -> PageMilli {
    let (low, high) = (PageMilli::of_space(first), PageMilli::of_space(last));
    if p < low {
        banded(low, p)
    } else if p > high {
        banded(high, p)
    } else {
        p
    }
}

/// `p`, which is past `bound`, drawn a quarter as far past it.
fn banded(bound: PageMilli, p: PageMilli) -> PageMilli {
    let past = div_nearest(i64::from(p.0) - i64::from(bound.0), 4);
    PageMilli(
        bound
            .0
            .saturating_add(saturate(past).clamp(-BAND_CAP, BAND_CAP)),
    )
}

/// `num / den` rounded to the nearest whole number, halves away from zero; `den` is positive.
pub(super) fn div_nearest(num: i64, den: i64) -> i64 {
    let half = den / 2;
    match num.signum() {
        -1 => -((-num + half) / den),
        _ => (num + half) / den,
    }
}

/// `v` as an `i32`, the nearest one when it does not fit.
pub(super) fn saturate(v: i64) -> i32 {
    i32::try_from(v.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).unwrap_or_default()
}
