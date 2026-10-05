//! A timed pure state machine. A [`Machine`] decides `(state, input, time, settings, outside
//! facts) -> (state, outputs)` and says when it wants to be stepped again with no input. It never
//! reads a clock and never does anything: `ds::machine::use_machine` keeps the state (or the
//! caller does, with `use_machine_in`), stamps every step with a
//! [`FrameClock`](crate::time::stamp::FrameClock), sleeps until [`Machine::wake`] on
//! `ds_core::time`'s clock (so a harness's virtual clock drives it), and hands each output to
//! the surface's effect handler.
//!
//! Time is [`Stamp`], the one "when" of the design system (milliseconds from an origin the
//! caller keeps); every timed machine in the pure crates (`ds-behaviour`, `ds-motion`) takes it.

use crate::time::stamp::Stamp;

/// What a machine is woken with when the time it asked for (its [`Machine::wake`]) has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Elapsed;

/// A state that changes over time or by input.
///
/// There is no `Default` bound: the caller supplies the first state. A deadline lives in the state
/// (`Armed { until }`), because [`Machine::wake`] reads the state alone.
pub trait Machine: Clone + PartialEq + 'static {
    /// What moves the machine; it can be woken by the clock alone.
    type In: From<Elapsed>;
    /// What the machine wants done.
    type Out: 'static;
    /// What the surface's settings say about timing; a change applies from the next step.
    type Params: Clone + PartialEq + 'static;
    /// Facts from outside the machine that a step reads to decide (the switcher's list of running
    /// apps, a roster's measured row heights): read-only and the caller's, not settings. `()` for
    /// a machine that needs none.
    type Ctx: 'static;

    /// The machine after `input` at `at`, and what it wants done. `cx` is read at this step: the
    /// caller provides it afresh each time, including for the step a wake causes.
    fn step(
        self,
        input: Self::In,
        at: Stamp,
        params: &Self::Params,
        cx: &Self::Ctx,
    ) -> (Self, Vec<Self::Out>);

    /// When to step again with no input; none when the machine is at rest, so an idle machine
    /// runs no timer at all.
    fn wake(&self) -> Option<Stamp>;
}

#[cfg(test)]
mod tests {
    use super::{Elapsed, Machine};
    use crate::time::stamp::Stamp;

    /// A toy: a lamp that turns itself off `hold` ms after the last press.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Lamp {
        Off,
        On { until: Stamp },
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
        type Ctx = ();

        fn step(self, input: In, at: Stamp, hold: &u64, _: &()) -> (Lamp, Vec<Out>) {
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
            let (next, out) = from.step(*input, Stamp(*at), &500, &());
            assert_eq!(next, *state, "{name}: state");
            assert_eq!(out.as_slice(), *outs, "{name}: outputs");
            assert_eq!(next.wake(), *wake, "{name}: wake");
        }
    }

    /// A toy that reads a fact from outside: a doorbell that rings only while the house is
    /// occupied, as the caller says at each step, and stops asking for a wake once it has rung.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Bell {
        Quiet,
        Pressed { until: Stamp },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Occupancy {
        Home,
        Away,
    }

    impl Machine for Bell {
        type In = In;
        type Out = Out;
        type Params = u64;
        type Ctx = Occupancy;

        fn step(self, input: In, at: Stamp, delay: &u64, at_home: &Occupancy) -> (Bell, Vec<Out>) {
            match (self, input, at_home) {
                (Bell::Quiet, In::Press, _) => (
                    Bell::Pressed {
                        until: at.after(*delay),
                    },
                    vec![],
                ),
                (Bell::Pressed { until }, In::Elapsed, Occupancy::Home) if at >= until => {
                    (Bell::Quiet, vec![Out::Light(true)])
                }
                (Bell::Pressed { until }, In::Elapsed, Occupancy::Away) if at >= until => {
                    (Bell::Quiet, vec![])
                }
                (state, _, _) => (state, vec![]),
            }
        }

        fn wake(&self) -> Option<Stamp> {
            match self {
                Bell::Quiet => None,
                Bell::Pressed { until } => Some(*until),
            }
        }
    }

    #[test]
    fn a_step_reads_the_context_it_is_given() {
        let pressed = Bell::Pressed { until: Stamp(300) };
        #[rustfmt::skip]
        let cases = [
            ("home: rings",         Occupancy::Home, &[Out::Light(true)][..]),
            ("away: stays silent",  Occupancy::Away, &[][..]),
        ];
        for (name, cx, outs) in cases {
            let (next, out) = pressed.step(In::Elapsed, Stamp(300), &300, &cx);
            assert_eq!(next, Bell::Quiet, "{name}: state");
            assert_eq!(out.as_slice(), outs, "{name}: outputs");
            assert_eq!(next.wake(), None, "{name}: at rest");
        }
    }

    #[test]
    fn a_machine_runs_to_rest_on_elapsed_alone() {
        let (mut lamp, mut at) = (Lamp::Off.step(In::Press, Stamp(10), &500, &()).0, Stamp(10));
        while let Some(due) = lamp.wake() {
            at = due;
            lamp = lamp.step(In::from(Elapsed), at, &500, &()).0;
        }
        assert_eq!((lamp, at), (Lamp::Off, Stamp(510)));
    }
}
