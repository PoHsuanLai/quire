//! Counting a quick run of presses (double and triple click). The surface keeps the press's
//! default action from running (it would start Blitz's own text selection), so it cannot use
//! the renderer's click count and keeps its own: within 500 ms and 4 px of the last press.

use crate::geometry::Point;
use std::time::{Duration, Instant};

/// Which press of a quick run this is: 1, 2, 3, then 1 again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Clicks(pub u8);

/// The previous press, as the next one is compared with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LastPress {
    pub(crate) at: Point,
    pub(crate) when: Instant,
    pub(crate) clicks: Clicks,
}

/// Longest gap between presses of one run.
const RUN_GAP: Duration = Duration::from_millis(500);
/// Farthest a press may land from the last one and still continue the run, in logical pixels.
const RUN_SLOP: f32 = 4.0;

/// The count of a press at `at` at `when`, after `last`.
pub(crate) fn clicks_after(last: Option<LastPress>, at: Point, when: Instant) -> Clicks {
    match last {
        Some(last)
            if when.saturating_duration_since(last.when) <= RUN_GAP
                && (at.x.0 - last.at.x.0).abs() <= RUN_SLOP
                && (at.y.0 - last.at.y.0).abs() <= RUN_SLOP =>
        {
            Clicks(last.clicks.0 % 3 + 1)
        }
        _ => Clicks(1),
    }
}

#[cfg(test)]
mod tests {
    use super::{Clicks, LastPress, clicks_after};
    use crate::geometry::{Point, Px};
    use std::time::{Duration, Instant};

    fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    /// (ms after the last press, where, the last press's count, expected count)
    const CASES: &[(u64, (f32, f32), u8, u8)] = &[
        (100, (10.0, 10.0), 1, 2),
        (100, (12.0, 13.0), 2, 3),
        (100, (10.0, 10.0), 3, 1),
        (600, (10.0, 10.0), 1, 1),
        (100, (20.0, 10.0), 1, 1),
    ];

    #[test]
    fn a_quick_nearby_press_continues_the_run() {
        let start = Instant::now();
        for &(gap, (x, y), before, expected) in CASES {
            let last = LastPress {
                at: at(10.0, 10.0),
                when: start,
                clicks: Clicks(before),
            };
            let got = clicks_after(Some(last), at(x, y), start + Duration::from_millis(gap));
            assert_eq!(
                got,
                Clicks(expected),
                "{gap} ms, ({x}, {y}), after {before}"
            );
        }
        assert_eq!(clicks_after(None, at(0.0, 0.0), start), Clicks(1));
    }
}
