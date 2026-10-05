//! The pure transition of the orb's turn: a [`Machine`] whose wake is the next frame while the
//! glows turn and nothing while they stand.

use super::model::Turn;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::FRAME_TICK;
use ds_core::time::stamp::Stamp;
use std::time::Duration;

/// Where the glows are `elapsed` after they stood at `from`, turning once per `period`. A
/// period of nothing holds them where they are.
pub(crate) fn turn_after(from: Turn, elapsed: Duration, period: Duration) -> Turn {
    if period.is_zero() {
        return from;
    }
    let through = elapsed.as_nanos() % period.as_nanos();
    let advance = through * u128::from(Turn::FULL) / period.as_nanos();
    // `advance` is below `Turn::FULL`, so it fits.
    let advance = u32::try_from(advance).unwrap_or_default();
    Turn((from.0 % Turn::FULL + advance) % Turn::FULL)
}

/// How the glows turn, from where they stood when it began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Run {
    from: Turn,
    since: Stamp,
    period: Duration,
    /// When the next frame is due.
    next: Stamp,
}

/// Where the glows are, and the run that moves them, if there is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Glow {
    pub(crate) turn: Turn,
    run: Option<Run>,
}

/// What moves the glows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlowIn {
    /// Turn once per this period from where they stand, or hold where they stand (`None`). The
    /// period already running again changes nothing.
    Spin(Option<Duration>),
    /// A frame is due.
    Elapsed,
}

impl From<Elapsed> for GlowIn {
    fn from(_: Elapsed) -> Self {
        GlowIn::Elapsed
    }
}

impl Machine for Glow {
    type In = GlowIn;
    type Out = ();
    type Params = ();
    type Ctx = ();

    fn step(self, input: GlowIn, at: Stamp, _: &(), _: &()) -> (Glow, Vec<()>) {
        let next = match (input, self.run) {
            (GlowIn::Spin(Some(period)), Some(run)) if run.period == period => self,
            (GlowIn::Spin(Some(period)), _) => Glow {
                run: Some(Run {
                    from: self.turn,
                    since: at,
                    period,
                    next: at.after_span(FRAME_TICK),
                }),
                ..self
            },
            (GlowIn::Spin(None), _) => Glow { run: None, ..self },
            (GlowIn::Elapsed, Some(run)) if at >= run.next => Glow {
                turn: turn_after(
                    run.from,
                    Duration::from_millis(at.since(run.since)),
                    run.period,
                ),
                run: Some(Run {
                    next: at.after_span(FRAME_TICK),
                    ..run
                }),
            },
            (GlowIn::Elapsed, _) => self,
        };
        (next, Vec::new())
    }

    fn wake(&self) -> Option<Stamp> {
        self.run.map(|run| run.next)
    }
}

#[cfg(test)]
mod tests {
    use super::{Glow, GlowIn, Run, Turn};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use std::time::Duration;

    const PERIOD: Duration = Duration::from_secs(20);

    fn turning(turn: u32, from: u32, since: u64, next: u64) -> Glow {
        Glow {
            turn: Turn(turn),
            run: Some(Run {
                from: Turn(from),
                since: Stamp(since),
                period: PERIOD,
                next: Stamp(next),
            }),
        }
    }

    fn standing(turn: u32) -> Glow {
        Glow {
            turn: Turn(turn),
            run: None,
        }
    }

    /// Name, state before, input, time, state after, next wake.
    type Case = (&'static str, Glow, GlowIn, u64, Glow, Option<u64>);

    #[test]
    fn the_glows_turn_a_frame_at_a_time_while_asked_to_and_hold_otherwise() {
        let spin = GlowIn::Spin(Some(PERIOD));
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("asked to spin: the run starts, a frame on", standing(0), spin, 100, turning(0, 0, 100, 116), Some(116)),
            ("a frame: the turn follows the time since the run began", turning(0, 0, 100, 116), GlowIn::Elapsed, 116, turning(800, 0, 100, 132), Some(132)),
            ("a frame woken early does nothing", turning(0, 0, 100, 116), GlowIn::Elapsed, 110, turning(0, 0, 100, 116), Some(116)),
            ("asked to spin at the same period: unchanged", turning(800, 0, 100, 132), spin, 120, turning(800, 0, 100, 132), Some(132)),
            ("a new period restarts from where they stand", turning(800, 0, 100, 132), GlowIn::Spin(Some(Duration::from_secs(10))), 130, Glow { turn: Turn(800), run: Some(Run { from: Turn(800), since: Stamp(130), period: Duration::from_secs(10), next: Stamp(146) }) }, Some(146)),
            ("asked to hold: they stand where they are", turning(800, 0, 100, 132), GlowIn::Spin(None), 120, standing(800), None),
            ("asked to hold while standing: unchanged", standing(5), GlowIn::Spin(None), 120, standing(5), None),
            ("a frame while standing does nothing", standing(5), GlowIn::Elapsed, 120, standing(5), None),
            ("a restart begins from the held turn", standing(250_000), spin, 5000, turning(250_000, 250_000, 5000, 5016), Some(5016)),
        ];
        for (name, from, input, at, state, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &(), &());
            assert_eq!(next, *state, "{name}: state");
            assert!(out.is_empty(), "{name}: outputs");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}: wake");
        }
    }
}
