//! What a `Scrubber` is told and what it shows (design/30 section 2.1a): the buffered ranges, the
//! pose that says whether the pointer is on it, and the time its tooltip reads. Data and the pure
//! arithmetic only.

use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_motion::spring::Millis;

/// A stretch of the length that is already loaded, in thousandths of the length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferedRange {
    /// Where the stretch starts.
    pub from: Fraction,
    /// Where it ends.
    pub to: Fraction,
}

/// Whether the pointer is on the scrubber, `data-state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ScrubPose {
    /// The pointer is elsewhere: no tooltip, the thumb at rest.
    #[default]
    Idle,
    /// The pointer is over the bar: the tooltip reads the time under it.
    Hover,
    /// A press is held: the tooltip follows the drag and the thumb is large.
    Dragging,
}

/// `ranges` as the bands to draw: each held to 0..=1000, empty ones dropped, sorted by start,
/// and overlapping or touching ones joined, so a band is drawn once.
pub fn merged(ranges: &[BufferedRange]) -> Vec<BufferedRange> {
    let mut held: Vec<BufferedRange> = ranges
        .iter()
        .map(|range| BufferedRange {
            from: range.from.clamped(),
            to: range.to.clamped(),
        })
        .filter(|range| range.from < range.to)
        .collect();
    held.sort_by_key(|range| (range.from, range.to));
    let mut joined: Vec<BufferedRange> = Vec::with_capacity(held.len());
    for range in held {
        match joined.last_mut() {
            Some(last) if range.from <= last.to => last.to = last.to.max(range.to),
            Some(_) | None => joined.push(range),
        }
    }
    joined
}

/// The time `at` of the way through a recording `length` long: `m:ss`, or `h:mm:ss` from one
/// hour, seconds rounded down.
pub fn time_text(length: Millis, at: Fraction) -> String {
    let millis = u64::from(length.0) * u64::from(at.clamped().0) / 1000;
    let secs = millis / 1000;
    let (hours, minutes, seconds) = (secs / 3600, secs / 60 % 60, secs % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(from: u16, to: u16) -> BufferedRange {
        BufferedRange {
            from: Fraction(from),
            to: Fraction(to),
        }
    }

    #[test]
    fn ranges_are_held_sorted_and_joined() {
        // (name, given, drawn)
        let cases: Vec<(&str, Vec<BufferedRange>, Vec<BufferedRange>)> = vec![
            ("none", vec![], vec![]),
            ("one", vec![range(100, 300)], vec![range(100, 300)]),
            ("empty dropped", vec![range(300, 300)], vec![]),
            ("backwards dropped", vec![range(400, 200)], vec![]),
            (
                "held to the ends",
                vec![range(0, 4000)],
                vec![range(0, 1000)],
            ),
            ("past the end dropped", vec![range(1200, 1500)], vec![]),
            (
                "unsorted",
                vec![range(600, 700), range(100, 200)],
                vec![range(100, 200), range(600, 700)],
            ),
            (
                "overlapping join",
                vec![range(100, 400), range(300, 600)],
                vec![range(100, 600)],
            ),
            (
                "touching join",
                vec![range(100, 300), range(300, 500)],
                vec![range(100, 500)],
            ),
            (
                "contained",
                vec![range(100, 900), range(200, 300)],
                vec![range(100, 900)],
            ),
            (
                "apart stay apart",
                vec![range(100, 200), range(300, 400)],
                vec![range(100, 200), range(300, 400)],
            ),
        ];
        for (name, given, want) in cases {
            assert_eq!(merged(&given), want, "{name}");
        }
    }

    #[test]
    fn the_tooltip_reads_minutes_and_seconds_and_hours_from_one() {
        // (name, length in ms, fraction, text)
        const CASES: &[(&str, u32, u16, &str)] = &[
            ("start", 180_000, 0, "0:00"),
            ("end", 180_000, 1000, "3:00"),
            ("half", 180_000, 500, "1:30"),
            ("rounds down", 61_999, 1000, "1:01"),
            ("under a minute", 45_000, 1000, "0:45"),
            ("an hour", 3_600_000, 1000, "1:00:00"),
            ("past an hour", 7_322_000, 1000, "2:02:02"),
            ("held to the end", 180_000, 4000, "3:00"),
            ("no length", 0, 500, "0:00"),
        ];
        for &(name, length, at, want) in CASES {
            assert_eq!(time_text(Millis(length), Fraction(at)), want, "{name}");
        }
    }

    #[test]
    fn every_pose_has_its_word() {
        for pose in ScrubPose::ALL {
            assert_eq!(ScrubPose::parse(pose.slug()), Some(*pose));
        }
        assert_eq!(ScrubPose::Dragging.slug(), "dragging");
    }
}
