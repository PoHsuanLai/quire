//! The level control's machine, pure (the user's brief of 2026-09-25): the press, the drag that
//! follows the pointer, the rubber band past either end, and the keys. The component measures the
//! track and runs the effects; this decides.

use crate::components::track::fraction_at;
use crate::components::vocab::Fraction;
use crate::geometry::units::{Px, Rect};

/// How far the capsule stretches at most past an end, however far the pointer goes.
const STRETCH_MAX: Px = Px(6.0);

/// The grids the keys step on: sixteen coarse steps (a volume key's), sixty-four fine ones
/// (with Shift).
const COARSE: u16 = 16;
const FINE: u16 = 64;

/// Whether the pointer holds the control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Hold {
    /// Not pressed.
    Idle,
    /// Pressed, the track not measured yet (it is read after layout, in a task); the pointer is
    /// at `x`, which a move keeps up to date until the measurement lands.
    Pressing { x: Px },
    /// Pressed on the track measured at `track`: the fill follows the pointer with no easing.
    Held { track: Rect },
}

/// The rubber band: how far the capsule stretches past an end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Stretch {
    /// At rest.
    None,
    /// Past the start (left) by this much.
    Start(Px),
    /// Past the end (right) by this much.
    End(Px),
}

impl Stretch {
    /// `data-over` and the stretch in pixels for `--rb`, or `None` at rest.
    pub(crate) fn attrs(self) -> Option<(&'static str, f32)> {
        match self {
            Stretch::None => None,
            Stretch::Start(Px(by)) => Some(("start", by)),
            Stretch::End(Px(by)) => Some(("end", by)),
        }
    }
}

/// Whether the rubber band plays: off under Reduced motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rubber {
    On,
    Off,
}

/// Which way a key moves the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nudge {
    Down,
    Up,
}

/// How far one key press moves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyStep {
    /// A sixteenth.
    Coarse,
    /// A sixty-fourth (Shift).
    Fine,
}

/// The control's state between renders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LevelState {
    pub hold: Hold,
    pub stretch: Stretch,
}

impl LevelState {
    /// At rest.
    pub(crate) const IDLE: LevelState = LevelState {
        hold: Hold::Idle,
        stretch: Stretch::None,
    };
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LevelInput {
    /// The pointer went down at `x`.
    Down { x: Px },
    /// The track under the press measured at `track`: the level goes to where the pointer is
    /// now. A press already let go (a click faster than the measurement) still sets the level at
    /// `x`, where it went down, and holds nothing.
    Measured { x: Px, track: Rect },
    /// The pointer moved to `x`.
    Move { x: Px },
    /// The pointer went up (or left the control).
    Up,
    /// A key.
    Key { nudge: Nudge, step: KeyStep },
}

/// A step's result: the next state, and the value to report, if it changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Stepped {
    pub state: LevelState,
    pub value: Option<Fraction>,
}

/// One step from `state` with the level at `value`.
pub(crate) fn step(
    state: LevelState,
    value: Fraction,
    input: LevelInput,
    rubber: Rubber,
) -> Stepped {
    let reported = |next: Fraction| (next != value).then_some(next);
    let held = |track: Rect, x: Px| Stepped {
        state: LevelState {
            hold: Hold::Held { track },
            stretch: stretch(track, x, rubber),
        },
        value: reported(fraction_at(track, x)),
    };
    match (state.hold, input) {
        (_, LevelInput::Down { x }) => Stepped {
            state: LevelState {
                hold: Hold::Pressing { x },
                stretch: Stretch::None,
            },
            value: None,
        },
        (Hold::Idle, LevelInput::Measured { x, track }) => Stepped {
            state,
            value: reported(fraction_at(track, x)),
        },
        (Hold::Pressing { x }, LevelInput::Measured { track, .. })
        | (Hold::Held { .. }, LevelInput::Measured { x, track })
        | (Hold::Held { track }, LevelInput::Move { x }) => held(track, x),
        (Hold::Pressing { .. }, LevelInput::Move { x }) => Stepped {
            state: LevelState {
                hold: Hold::Pressing { x },
                ..state
            },
            value: None,
        },
        (Hold::Idle, LevelInput::Move { .. }) => Stepped { state, value: None },
        (_, LevelInput::Up) => Stepped {
            state: LevelState::IDLE,
            value: None,
        },
        (_, LevelInput::Key { nudge, step }) => Stepped {
            state,
            value: reported(keyed(value, nudge, step)),
        },
    }
}

/// The rubber band for the pointer at `x` on `track`: past an end, the capsule stretches by
/// `max x d / (d + 2 max)`, which follows a small overshoot almost 1:3 and never passes `max`.
pub(crate) fn stretch(track: Rect, x: Px, rubber: Rubber) -> Stretch {
    let band = |past: f32| Px(STRETCH_MAX.0 * past / (past + 2.0 * STRETCH_MAX.0));
    let (left, right) = (track.left().0, track.left().0 + track.size.width.0);
    match rubber {
        Rubber::Off => Stretch::None,
        Rubber::On if x.0 < left => Stretch::Start(band(left - x.0)),
        Rubber::On if x.0 > right => Stretch::End(band(x.0 - right)),
        Rubber::On => Stretch::None,
    }
}

/// The level one key press away: the next point of the step's grid above or below `value`, so
/// a level between two points lands on one rather than keeping its offset.
pub(crate) fn keyed(value: Fraction, nudge: Nudge, step: KeyStep) -> Fraction {
    let steps = match step {
        KeyStep::Coarse => COARSE,
        KeyStep::Fine => FINE,
    };
    let now = value.clamped().0;
    let point =
        |k: u16| Fraction(((u32::from(k) * 1000 + u32::from(steps) / 2) / u32::from(steps)) as u16);
    let found = match nudge {
        Nudge::Up => (0..=steps).map(point).find(|p| p.0 > now),
        Nudge::Down => (0..=steps).rev().map(point).find(|p| p.0 < now),
    };
    found.unwrap_or(Fraction(now))
}

/// Whether a change from `old` to `new` crossed one of the sixteen steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Crossing {
    Within,
    Crossed,
}

/// Which of the sixteen steps `value` is in.
pub(crate) fn step_of(value: Fraction) -> u16 {
    (value.clamped().0 * COARSE / 1000).min(COARSE - 1)
}

/// Whether `old` to `new` crossed a step.
pub(crate) fn crossing(old: Fraction, new: Fraction) -> Crossing {
    if step_of(old) == step_of(new) {
        Crossing::Within
    } else {
        Crossing::Crossed
    }
}

#[cfg(test)]
#[path = "machine_tests.rs"]
mod tests;
