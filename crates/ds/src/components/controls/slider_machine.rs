//! The capsule slider's machine, pure (the user's brief of 2026-09-25): the press, the drag that
//! follows the pointer, and the keys. The component measures the track and runs the effects; this
//! decides.

use crate::components::controls::track::fraction_at;
use ds_core::geometry::units::{Px, Rect};
use ds_core::vocab::{Fraction, PressPhase};

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

impl Hold {
    /// The press this holds, as the shared vocabulary says it: down on the track, measured or not.
    pub(crate) fn phase(self) -> PressPhase {
        match self {
            Hold::Idle => PressPhase::Idle,
            Hold::Pressing { .. } | Hold::Held { .. } => PressPhase::Pressed,
        }
    }
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
    pub state: Hold,
    pub value: Option<Fraction>,
}

/// One step from `state` with the level at `value`.
pub(crate) fn step(state: Hold, value: Fraction, input: LevelInput) -> Stepped {
    let reported = |next: Fraction| (next != value).then_some(next);
    let held = |track: Rect, x: Px| Stepped {
        state: Hold::Held { track },
        value: reported(fraction_at(track, x)),
    };
    match (state, input) {
        (_, LevelInput::Down { x }) => Stepped {
            state: Hold::Pressing { x },
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
            state: Hold::Pressing { x },
            value: None,
        },
        (Hold::Idle, LevelInput::Move { .. }) => Stepped { state, value: None },
        (_, LevelInput::Up) => Stepped {
            state: Hold::Idle,
            value: None,
        },
        (_, LevelInput::Key { nudge, step }) => Stepped {
            state,
            value: reported(keyed(value, nudge, step)),
        },
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

#[cfg(test)]
#[path = "slider_machine_tests.rs"]
mod tests;
