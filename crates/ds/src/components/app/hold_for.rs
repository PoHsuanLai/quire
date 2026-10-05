//! A one-shot "for this long" timer as a [`Machine`]: started, it holds for a delay, then is
//! done. The toast-like pills use it to show a word for [`DelayToken::ToastHold`] (the link pill's
//! "Copied") or to slide away when the hold ends (the send pill's "Sent"). The end is in the state,
//! so a start while it holds simply moves the end out, and nothing is left to cancel.
//!
//! [`DelayToken::ToastHold`]: ds_style::tokens::delay::DelayToken::ToastHold

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_motion::machine::{MachineRef, use_machine};
use std::time::Duration;

/// Where the hold is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldFor {
    /// Never started.
    Rest,
    /// Holding until this time.
    Held(Stamp),
    /// The last start's hold has run out.
    Done,
}

/// What moves the hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldForIn {
    /// Hold for the delay from now, from any state.
    Start,
    /// The hold's end came.
    Elapsed,
}

impl From<Elapsed> for HoldForIn {
    fn from(_: Elapsed) -> Self {
        HoldForIn::Elapsed
    }
}

impl Machine for HoldFor {
    type In = HoldForIn;
    type Out = ();
    /// How long a start holds.
    type Params = Duration;
    type Ctx = ();

    fn step(self, input: HoldForIn, at: Stamp, delay: &Duration, _: &()) -> (HoldFor, Vec<()>) {
        let next = match (self, input) {
            (_, HoldForIn::Start) => HoldFor::Held(at.after_span(*delay)),
            (HoldFor::Held(until), HoldForIn::Elapsed) if at >= until => HoldFor::Done,
            (state, HoldForIn::Elapsed) => state,
        };
        (next, Vec::new())
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            HoldFor::Held(until) => Some(*until),
            HoldFor::Rest | HoldFor::Done => None,
        }
    }
}

/// When a [`use_hold_for`] hold starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Begin {
    /// As the component mounts.
    AtMount,
    /// When asked to.
    OnStart,
}

/// A hold of `delay` (a `DelayToken`'s delay, or a frame), started as `begin` says.
pub(crate) fn use_hold_for(delay: Duration, begin: Begin) -> MachineRef<HoldFor> {
    use_machine(
        move |at| match begin {
            Begin::AtMount => HoldFor::Held(at.after_span(delay)),
            Begin::OnStart => HoldFor::Rest,
        },
        delay,
        || (),
        |(), _| {},
    )
}

#[cfg(test)]
mod tests {
    use super::{HoldFor, HoldForIn};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use std::time::Duration;

    #[test]
    fn a_start_holds_for_the_delay_then_it_is_done() {
        let delay = Duration::from_millis(5000);
        let held = |at| HoldFor::Held(Stamp(at));
        /// Name, state before, input, time, state after, next wake.
        type Case = (&'static str, HoldFor, HoldForIn, u64, HoldFor, Option<u64>);
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("a start from rest holds", HoldFor::Rest, HoldForIn::Start, 100, held(5100), Some(5100)),
            ("a start while holding moves the end out", held(5100), HoldForIn::Start, 3000, held(8000), Some(8000)),
            ("a start after it ran out holds again", HoldFor::Done, HoldForIn::Start, 9000, held(14000), Some(14000)),
            ("woken early: still holding", held(5100), HoldForIn::Elapsed, 5099, held(5100), Some(5100)),
            ("the end: done", held(5100), HoldForIn::Elapsed, 5100, HoldFor::Done, None),
            ("a wake at rest: nothing", HoldFor::Rest, HoldForIn::Elapsed, 6000, HoldFor::Rest, None),
            ("a wake when done: nothing", HoldFor::Done, HoldForIn::Elapsed, 6000, HoldFor::Done, None),
        ];
        for (name, from, input, at, state, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &delay, &());
            assert_eq!(next, *state, "{name}: state");
            assert!(out.is_empty(), "{name}: outputs");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}: wake");
        }
    }
}
