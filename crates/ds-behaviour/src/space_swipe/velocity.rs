//! The lift's velocity: a least-squares line through the last 80 ms of the fingers' positions.

use ds_core::time::stamp::Stamp;

use super::model::{PageMilli, PagesPerSecMilli, Samples};
use super::numbers::{div_nearest, saturate};

/// How far back from the lift a position still says how fast the fingers were going.
const WINDOW_MS: i64 = 80;

/// `samples` with `p` seen at `at` added and anything older than the window dropped.
pub(super) fn record(samples: Samples, at: Stamp, p: PageMilli) -> Samples {
    let kept = samples
        .0
        .into_iter()
        .filter(|(seen, _)| within_window(at, *seen))
        .chain([(at, p)])
        .collect();
    Samples(kept)
}

/// The fingers' speed at a lift at `now`; zero with fewer than two samples in the window.
pub(super) fn lift_velocity(samples: &Samples, now: Stamp) -> PagesPerSecMilli {
    let points: Vec<(i64, i64)> = samples
        .0
        .iter()
        .filter(|(seen, _)| within_window(now, *seen))
        .map(|(seen, p)| (-age(now, *seen), i64::from(p.0)))
        .collect();
    PagesPerSecMilli(saturate(slope_per_second(&points)))
}

/// How long before `now` a sample was taken, in milliseconds.
fn age(now: Stamp, seen: Stamp) -> i64 {
    i64::try_from(now.since(seen)).unwrap_or(i64::MAX)
}

/// Whether a sample taken at `seen` is still in the window at `now`.
fn within_window(now: Stamp, seen: Stamp) -> bool {
    age(now, seen) <= WINDOW_MS
}

/// The slope of the least-squares line through `(ms, position)` points, per second; zero when
/// the points do not span two different times.
fn slope_per_second(points: &[(i64, i64)]) -> i64 {
    let n = i64::try_from(points.len()).unwrap_or(i64::MAX);
    let sum = |f: fn(&(i64, i64)) -> i64| points.iter().map(f).sum::<i64>();
    let (t, x) = (sum(|p| p.0), sum(|p| p.1));
    let (tt, tx) = (sum(|p| p.0 * p.0), sum(|p| p.0 * p.1));
    let spread = n * tt - t * t;
    match spread {
        0 => 0,
        _ => div_nearest((n * tx - t * x) * 1000, spread),
    }
}
