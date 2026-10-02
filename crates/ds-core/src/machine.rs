//! A timed pure state machine. A [`Machine`] decides `(state, input, time) -> (state, outputs)`
//! and says when it wants to be stepped again with no input. It never reads a clock and never
//! does anything: `ds::machine::use_machine` owns the state, stamps every step with a
//! [`FrameClock`](crate::time::stamp::FrameClock), sleeps until [`Machine::wake`] on
//! `ds_core::time`'s clock (so a harness's virtual clock drives it), and hands each output to
//! the surface's effect handler.
//!
//! Time is [`Stamp`], the one "when" of the design system (milliseconds from an origin the
//! caller keeps); `ds-motion`'s gesture machines take the same type.

use crate::time::stamp::Stamp;

/// What a machine is woken with when the time it asked for (its [`Machine::wake`]) has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Elapsed;

/// A state that changes over time or by input.
pub trait Machine: Clone + PartialEq + Default + 'static {
    /// What moves the machine; it can be woken by the clock alone.
    type In: From<Elapsed>;
    /// What the machine wants done.
    type Out: 'static;
    /// What the surface's settings say about timing; a change applies from the next step.
    type Params: Clone + PartialEq + 'static;

    /// The machine after `input` at `at`, and what it wants done.
    fn step(self, input: Self::In, at: Stamp, params: &Self::Params) -> (Self, Vec<Self::Out>);

    /// When to step again with no input; none when the machine is at rest, so an idle machine
    /// runs no timer at all.
    fn wake(&self) -> Option<Stamp>;
}

#[cfg(test)]
mod tests {
    use super::{Elapsed, Machine};
    use crate::time::stamp::Stamp;

    /// A toy: a lamp that turns itself off `hold` ms after the last press.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    enum Lamp {
        #[default]
        Off,
        On {
            until: Stamp,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum In {
        Press,
        Elapsed,
    }

    impl From<Elapsed> for In {
        fn from(_: Elapsed) -> In {
            In::Elapsed
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Out {
        Light(bool),
    }

    impl Machine for Lamp {
        type In = In;
        type Out = Out;
        type Params = u64;

        fn step(self, input: In, at: Stamp, hold: &u64) -> (Lamp, Vec<Out>) {
            match (self, input) {
                (_, In::Press) => (
                    Lamp::On {
                        until: at.after(*hold),
                    },
                    if self == Lamp::Off {
                        vec![Out::Light(true)]
                    } else {
                        vec![]
                    },
                ),
                (Lamp::On { until }, In::Elapsed) if at >= until => {
                    (Lamp::Off, vec![Out::Light(false)])
                }
                (state, In::Elapsed) => (state, vec![]),
            }
        }

        fn wake(&self) -> Option<Stamp> {
            match self {
                Lamp::Off => None,
                Lamp::On { until } => Some(*until),
            }
        }
    }

    /// Name, state before, input, time, state after, outputs, next wake.
    type Case = (
        &'static str,
        Lamp,
        In,
        u64,
        Lamp,
        &'static [Out],
        Option<Stamp>,
    );

    #[test]
    fn a_table_of_steps_and_wakes() {
        let on = |t| Lamp::On { until: Stamp(t) };
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("press lights the lamp",    Lamp::Off, In::Press,   100, on(600), &[Out::Light(true)],  Some(Stamp(600))),
            ("press extends the hold",   on(600),   In::Press,   300, on(800), &[],                  Some(Stamp(800))),
            ("early elapsed does nothing", on(600), In::Elapsed, 599, on(600), &[],                  Some(Stamp(600))),
            ("elapsed at the wake turns it off", on(600), In::Elapsed, 600, Lamp::Off, &[Out::Light(false)], None),
            ("elapsed at rest is nothing", Lamp::Off, In::Elapsed, 50, Lamp::Off, &[],               None),
        ];
        for (name, from, input, at, state, outs, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &500);
            assert_eq!(next, *state, "{name}: state");
            assert_eq!(out.as_slice(), *outs, "{name}: outputs");
            assert_eq!(next.wake(), *wake, "{name}: wake");
        }
    }
}
