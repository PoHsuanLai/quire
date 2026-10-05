//! A held press on a stepper (design/30 section 2.1: hold repeats) as a pure [`Machine`]: a press
//! steps at once, then repeats after the start delay and at the repeat pitch until it is let go.
//! The next step's time is in the state, so a release leaves nothing to cancel: the machine has
//! no wake and the hook's timer stops.

use crate::components::fields::stepper::model::StepDirection;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_style::tokens::delay::DelayToken;
use std::time::Duration;

/// When a held press repeats: how long it waits to start and the pitch after.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RepeatPace {
    pub(crate) start: Duration,
    pub(crate) every: Duration,
}

impl Default for RepeatPace {
    /// `DelayToken::RepeatStart`, then `DelayToken::RepeatEvery`.
    fn default() -> Self {
        RepeatPace {
            start: DelayToken::RepeatStart.delay(),
            every: DelayToken::RepeatEvery.delay(),
        }
    }
}

/// The half held down, if any, and when it repeats next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepHold {
    /// No half is held.
    Idle,
    /// `direction` is held; it steps again at `due`.
    Held {
        direction: StepDirection,
        due: Stamp,
    },
}

/// What moves the hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldIn {
    /// A press on a half.
    Press(StepDirection),
    /// The press let go (or the pair lost the pointer or the focus).
    Release,
    /// The next repeat is due.
    Elapsed,
}

impl From<Elapsed> for HoldIn {
    fn from(_: Elapsed) -> Self {
        HoldIn::Elapsed
    }
}

impl StepHold {
    /// The half of the pair held down, as the pressed appearance wants it.
    pub(crate) fn direction(self) -> Option<StepDirection> {
        match self {
            StepHold::Idle => None,
            StepHold::Held { direction, .. } => Some(direction),
        }
    }
}

impl Machine for StepHold {
    type In = HoldIn;
    /// One step toward this direction.
    type Out = StepDirection;
    type Params = RepeatPace;
    type Ctx = ();

    /// A press steps once and waits `start`; each repeat steps and waits `every`. A repeat that is
    /// woken early, or after the press was let go, does nothing, so no press can repeat another's
    /// steps.
    fn step(
        self,
        input: HoldIn,
        at: Stamp,
        pace: &RepeatPace,
        _: &(),
    ) -> (StepHold, Vec<StepDirection>) {
        match (self, input) {
            (_, HoldIn::Press(direction)) => (
                StepHold::Held {
                    direction,
                    due: at.after_span(pace.start),
                },
                vec![direction],
            ),
            (_, HoldIn::Release) => (StepHold::Idle, Vec::new()),
            (StepHold::Held { direction, due }, HoldIn::Elapsed) if at >= due => (
                StepHold::Held {
                    direction,
                    due: at.after_span(pace.every),
                },
                vec![direction],
            ),
            (state, HoldIn::Elapsed) => (state, Vec::new()),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            StepHold::Idle => None,
            StepHold::Held { due, .. } => Some(*due),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HoldIn, RepeatPace, StepHold};
    use crate::components::fields::stepper::model::StepDirection::{Down, Up};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use ds_style::tokens::delay::DelayToken;

    fn held(
        direction: crate::components::fields::stepper::model::StepDirection,
        due: u64,
    ) -> StepHold {
        StepHold::Held {
            direction,
            due: Stamp(due),
        }
    }

    /// Name, state before, input, time, state after, steps, next wake.
    type Case = (
        &'static str,
        StepHold,
        HoldIn,
        u64,
        StepHold,
        &'static [crate::components::fields::stepper::model::StepDirection],
        Option<u64>,
    );

    #[test]
    fn a_press_steps_at_once_then_repeats_on_the_pace_until_it_is_let_go() {
        let pace = RepeatPace::default();
        assert_eq!(pace.start, DelayToken::RepeatStart.delay());
        assert_eq!(pace.every, DelayToken::RepeatEvery.delay());
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("a press steps and waits the start delay", StepHold::Idle, HoldIn::Press(Up), 100, held(Up, 600), &[Up], Some(600)),
            ("a press on the other half replaces it", held(Up, 600), HoldIn::Press(Down), 300, held(Down, 800), &[Down], Some(800)),
            ("woken early: nothing", held(Up, 600), HoldIn::Elapsed, 599, held(Up, 600), &[], Some(600)),
            ("the start delay ended: a repeat", held(Up, 600), HoldIn::Elapsed, 600, held(Up, 670), &[Up], Some(670)),
            ("each repeat waits the pitch from when it ran", held(Up, 670), HoldIn::Elapsed, 675, held(Up, 745), &[Up], Some(745)),
            ("a release stops it", held(Up, 670), HoldIn::Release, 650, StepHold::Idle, &[], None),
            ("a repeat after the release does nothing", StepHold::Idle, HoldIn::Elapsed, 670, StepHold::Idle, &[], None),
            ("a release with nothing held does nothing", StepHold::Idle, HoldIn::Release, 5, StepHold::Idle, &[], None),
        ];
        for (name, from, input, at, state, steps, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &pace, &());
            assert_eq!(next, *state, "{name}: state");
            assert_eq!(out.as_slice(), *steps, "{name}: steps");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}: wake");
        }
    }

    #[test]
    fn the_pressed_half_is_the_held_one() {
        assert_eq!(StepHold::Idle.direction(), None);
        assert_eq!(held(Down, 9).direction(), Some(Down));
    }
}
