//! The scrubber's machine, pure: where the pointer is (over the bar, or holding a press), and
//! what each input reports. The component measures the bar and carries the reports out; this
//! decides. A press that goes down on a bar measured at `track` holds that measurement until it
//! is let go, so a drag that leaves the bar still reads positions along it.

use crate::components::controls::scrubber_model::ScrubPose;
use crate::components::controls::track::fraction_at;
use ds_core::geometry::units::{Px, Rect};
use ds_core::vocab::Fraction;

/// A key's step: a fiftieth of the length, a tenth with Shift.
const SMALL: u16 = 20;
const LARGE: u16 = 100;

/// Where the pointer is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Pointer {
    /// Not on the bar.
    #[default]
    Away,
    /// Over the bar, at `at` along it.
    Over { track: Rect, at: Fraction },
    /// Holding a press, at `at` along the bar.
    Held { track: Rect, at: Fraction },
}

impl Pointer {
    /// The pose to draw, and where along the bar the tooltip stands.
    pub(crate) fn pose(self) -> (ScrubPose, Fraction) {
        match self {
            Pointer::Away => (ScrubPose::Idle, Fraction(0)),
            Pointer::Over { at, .. } => (ScrubPose::Hover, at),
            Pointer::Held { at, .. } => (ScrubPose::Dragging, at),
        }
    }
}

/// Which way a key moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Jump {
    /// A small step back.
    Back,
    /// A small step on.
    Forward,
    /// A large step back (Shift).
    BackFar,
    /// A large step on (Shift).
    ForwardFar,
    /// To the start.
    Start,
    /// To the end.
    End,
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScrubInput {
    /// The pointer moved over the bar, which is `track` as last measured.
    Moved { track: Rect, x: Px },
    /// The pointer left the bar.
    Left,
    /// The primary button went down on the bar measured at `track`.
    Down { track: Rect, x: Px },
    /// The pointer moved with the press held (the host follows it past the bar's box).
    Dragged { x: Px },
    /// The button came up.
    Up { x: Px },
    /// The drag was abandoned (Escape, or the pointer taken away).
    Cancel,
    /// A key.
    Key(Jump),
}

/// What the machine tells its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrubOut {
    /// A press began at this place.
    Start(Fraction),
    /// The drag moved to this place.
    Scrub(Fraction),
    /// The press ended at this place.
    End(Fraction),
    /// The press was abandoned.
    Cancel,
    /// A key asks for this place.
    Seek(Fraction),
}

/// One step from `pointer` with the position at `position`.
pub(crate) fn step(
    pointer: Pointer,
    position: Fraction,
    input: ScrubInput,
) -> (Pointer, Option<ScrubOut>) {
    match (pointer, input) {
        (Pointer::Held { track, at }, ScrubInput::Moved { x, .. } | ScrubInput::Dragged { x }) => {
            let to = fraction_at(track, x);
            (
                Pointer::Held { track, at: to },
                (to != at).then_some(ScrubOut::Scrub(to)),
            )
        }
        (Pointer::Away | Pointer::Over { .. }, ScrubInput::Moved { track, x }) => (
            Pointer::Over {
                track,
                at: fraction_at(track, x),
            },
            None,
        ),
        (Pointer::Away | Pointer::Over { .. }, ScrubInput::Dragged { .. }) => (pointer, None),
        (_, ScrubInput::Down { track, x }) => {
            let at = fraction_at(track, x);
            (Pointer::Held { track, at }, Some(ScrubOut::Start(at)))
        }
        (Pointer::Held { track, .. }, ScrubInput::Up { x }) => {
            let at = fraction_at(track, x);
            (Pointer::Over { track, at }, Some(ScrubOut::End(at)))
        }
        (Pointer::Away | Pointer::Over { .. }, ScrubInput::Up { .. }) => (pointer, None),
        (Pointer::Held { track, at }, ScrubInput::Cancel) => {
            (Pointer::Over { track, at }, Some(ScrubOut::Cancel))
        }
        (Pointer::Away | Pointer::Over { .. }, ScrubInput::Cancel) => (pointer, None),
        (Pointer::Over { .. }, ScrubInput::Left) => (Pointer::Away, None),
        (Pointer::Away | Pointer::Held { .. }, ScrubInput::Left) => (pointer, None),
        (Pointer::Held { .. }, ScrubInput::Key(_)) => (pointer, None),
        (Pointer::Away | Pointer::Over { .. }, ScrubInput::Key(jump)) => {
            let to = jumped(position, jump);
            (
                pointer,
                (to != position.clamped()).then_some(ScrubOut::Seek(to)),
            )
        }
    }
}

/// `position` moved by `jump`, held to 0..=1000.
pub(crate) fn jumped(position: Fraction, jump: Jump) -> Fraction {
    let now = position.clamped().0;
    Fraction(match jump {
        Jump::Back => now.saturating_sub(SMALL),
        Jump::BackFar => now.saturating_sub(LARGE),
        Jump::Forward => (now + SMALL).min(1000),
        Jump::ForwardFar => (now + LARGE).min(1000),
        Jump::Start => 0,
        Jump::End => 1000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds_core::geometry::units::{Point, Size};

    /// A bar 200 px wide whose left edge is at 100.
    fn bar() -> Rect {
        Rect {
            origin: Point {
                x: Px(100.0),
                y: Px(0.0),
            },
            size: Size {
                width: Px(200.0),
                height: Px(12.0),
            },
        }
    }

    const fn over(at: u16) -> Pointer {
        Pointer::Over {
            track: BAR,
            at: Fraction(at),
        }
    }

    const fn held(at: u16) -> Pointer {
        Pointer::Held {
            track: BAR,
            at: Fraction(at),
        }
    }

    const BAR: Rect = Rect {
        origin: Point {
            x: Px(100.0),
            y: Px(0.0),
        },
        size: Size {
            width: Px(200.0),
            height: Px(12.0),
        },
    };

    /// A row: name, from, position, input, to, report.
    type Case = (
        &'static str,
        Pointer,
        u16,
        ScrubInput,
        Pointer,
        Option<ScrubOut>,
    );

    #[test]
    fn each_input_has_its_next_pointer_and_report() {
        // (name, from, position, input, to, report)
        let cases: Vec<Case> = vec![
            (
                "moving over the bar hovers",
                Pointer::Away,
                0,
                ScrubInput::Moved {
                    track: bar(),
                    x: Px(200.0),
                },
                over(500),
                None,
            ),
            (
                "moving along the bar follows",
                over(500),
                0,
                ScrubInput::Moved {
                    track: bar(),
                    x: Px(250.0),
                },
                over(750),
                None,
            ),
            (
                "leaving the bar goes away",
                over(500),
                0,
                ScrubInput::Left,
                Pointer::Away,
                None,
            ),
            (
                "a press starts at the pointer",
                over(500),
                0,
                ScrubInput::Down {
                    track: bar(),
                    x: Px(200.0),
                },
                held(500),
                Some(ScrubOut::Start(Fraction(500))),
            ),
            (
                "a press with no hover first still starts",
                Pointer::Away,
                0,
                ScrubInput::Down {
                    track: bar(),
                    x: Px(100.0),
                },
                held(0),
                Some(ScrubOut::Start(Fraction(0))),
            ),
            (
                "a drag reports where it is",
                held(500),
                0,
                ScrubInput::Dragged { x: Px(250.0) },
                held(750),
                Some(ScrubOut::Scrub(Fraction(750))),
            ),
            (
                "a drag past the end holds at the end",
                held(500),
                0,
                ScrubInput::Dragged { x: Px(900.0) },
                held(1000),
                Some(ScrubOut::Scrub(Fraction(1000))),
            ),
            (
                "a drag before the start holds at the start",
                held(500),
                0,
                ScrubInput::Dragged { x: Px(-40.0) },
                held(0),
                Some(ScrubOut::Scrub(Fraction(0))),
            ),
            (
                "a drag that does not move reports nothing",
                held(500),
                0,
                ScrubInput::Dragged { x: Px(200.0) },
                held(500),
                None,
            ),
            (
                "a move while pressed is a drag",
                held(500),
                0,
                ScrubInput::Moved {
                    track: bar(),
                    x: Px(300.0),
                },
                held(1000),
                Some(ScrubOut::Scrub(Fraction(1000))),
            ),
            (
                "leaving while pressed keeps the press",
                held(500),
                0,
                ScrubInput::Left,
                held(500),
                None,
            ),
            (
                "letting go ends the press where it is",
                held(500),
                0,
                ScrubInput::Up { x: Px(150.0) },
                over(250),
                Some(ScrubOut::End(Fraction(250))),
            ),
            (
                "a release with no press reports nothing",
                over(500),
                0,
                ScrubInput::Up { x: Px(150.0) },
                over(500),
                None,
            ),
            (
                "escape abandons the press",
                held(500),
                0,
                ScrubInput::Cancel,
                over(500),
                Some(ScrubOut::Cancel),
            ),
            (
                "escape with no press does nothing",
                over(500),
                0,
                ScrubInput::Cancel,
                over(500),
                None,
            ),
            (
                "a drag report with no press is ignored",
                over(500),
                0,
                ScrubInput::Dragged { x: Px(250.0) },
                over(500),
                None,
            ),
            (
                "a key steps on",
                Pointer::Away,
                500,
                ScrubInput::Key(Jump::Forward),
                Pointer::Away,
                Some(ScrubOut::Seek(Fraction(520))),
            ),
            (
                "a key with Shift steps far back",
                Pointer::Away,
                500,
                ScrubInput::Key(Jump::BackFar),
                Pointer::Away,
                Some(ScrubOut::Seek(Fraction(400))),
            ),
            (
                "end at the end reports nothing",
                Pointer::Away,
                1000,
                ScrubInput::Key(Jump::End),
                Pointer::Away,
                None,
            ),
            (
                "a key during a press is ignored",
                held(500),
                500,
                ScrubInput::Key(Jump::Forward),
                held(500),
                None,
            ),
        ];
        for (name, from, position, input, to, report) in cases {
            let got = step(from, Fraction(position), input);
            assert_eq!(got, (to, report), "{name}");
        }
    }

    #[test]
    fn keys_stop_at_the_ends() {
        // (position, jump, lands)
        const CASES: &[(u16, Jump, u16)] = &[
            (10, Jump::Back, 0),
            (990, Jump::Forward, 1000),
            (50, Jump::BackFar, 0),
            (950, Jump::ForwardFar, 1000),
            (400, Jump::Start, 0),
            (400, Jump::End, 1000),
            (4000, Jump::Back, 980),
        ];
        for &(position, jump, want) in CASES {
            assert_eq!(
                jumped(Fraction(position), jump),
                Fraction(want),
                "{position} {jump:?}"
            );
        }
    }

    #[test]
    fn the_pose_follows_the_pointer() {
        assert_eq!(Pointer::Away.pose(), (ScrubPose::Idle, Fraction(0)));
        assert_eq!(over(300).pose(), (ScrubPose::Hover, Fraction(300)));
        assert_eq!(held(700).pose(), (ScrubPose::Dragging, Fraction(700)));
    }
}
