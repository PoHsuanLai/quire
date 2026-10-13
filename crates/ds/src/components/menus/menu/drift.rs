//! A menu's scroll arrow held under the pointer, as a pure [`Machine`]: the list scrolls one step
//! at once, then one more each `DelayToken::ScrollArrowStep` until the pointer leaves the arrow.
//! The next step's time is in the state, so leaving needs nothing cancelled.

use ds_core::geometry::units::Px;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use std::time::Duration;

/// How far the list scrolls at each step of a held arrow.
pub(crate) const SCROLL_ARROW_STEP: Px = Px(6.0);

/// Which end of the list an arrow is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edge {
    /// The top: it scrolls toward the first row.
    Start,
    /// The bottom: it scrolls toward the last row.
    End,
}

impl Edge {
    /// The strip's `data-end`.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Edge::Start => "start",
            Edge::End => "end",
        }
    }

    /// How far one step moves the list's offset.
    pub(crate) fn step(self) -> Px {
        match self {
            Edge::Start => Px(-SCROLL_ARROW_STEP.0),
            Edge::End => SCROLL_ARROW_STEP,
        }
    }
}

/// The arrow held, if any, and when it steps next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Drift {
    /// No arrow is held.
    Idle,
    /// The pointer is on the arrow at `edge`; it steps again at `due`.
    Toward { edge: Edge, due: Stamp },
}

/// What moves the drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DriftIn {
    /// The pointer came onto an arrow.
    Over(Edge),
    /// The pointer left the arrow, or there is nowhere further to scroll.
    Off,
    /// The next step is due.
    Elapsed,
}

impl From<Elapsed> for DriftIn {
    fn from(_: Elapsed) -> Self {
        DriftIn::Elapsed
    }
}

impl Machine for Drift {
    type In = DriftIn;
    /// One step toward this edge.
    type Out = Edge;
    /// The time between steps.
    type Params = Duration;
    type Ctx = ();

    fn step(self, input: DriftIn, at: Stamp, every: &Duration, _: &()) -> (Drift, Vec<Edge>) {
        match (self, input) {
            (_, DriftIn::Over(edge)) => (
                Drift::Toward {
                    edge,
                    due: at.after_span(*every),
                },
                vec![edge],
            ),
            (_, DriftIn::Off) => (Drift::Idle, Vec::new()),
            (Drift::Toward { edge, due }, DriftIn::Elapsed) if at >= due => (
                Drift::Toward {
                    edge,
                    due: at.after_span(*every),
                },
                vec![edge],
            ),
            (state, DriftIn::Elapsed) => (state, Vec::new()),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Drift::Idle => None,
            Drift::Toward { due, .. } => Some(*due),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Drift, DriftIn, Edge};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use std::time::Duration;

    const EVERY: Duration = Duration::from_millis(16);

    fn toward(edge: Edge, due: u64) -> Drift {
        Drift::Toward {
            edge,
            due: Stamp(due),
        }
    }

    #[test]
    fn a_held_arrow_steps_at_once_then_each_period_until_it_is_left() {
        // (name, state, input, time, state after, steps, wake)
        #[rustfmt::skip]
        let cases = [
            ("over steps at once", Drift::Idle, DriftIn::Over(Edge::End), 100, toward(Edge::End, 116), vec![Edge::End], Some(116)),
            ("early wake waits", toward(Edge::End, 116), DriftIn::Elapsed, 110, toward(Edge::End, 116), vec![], Some(116)),
            ("due steps again", toward(Edge::Start, 116), DriftIn::Elapsed, 116, toward(Edge::Start, 132), vec![Edge::Start], Some(132)),
            ("off stops", toward(Edge::End, 116), DriftIn::Off, 120, Drift::Idle, vec![], None),
            ("idle ignores a wake", Drift::Idle, DriftIn::Elapsed, 200, Drift::Idle, vec![], None),
        ];
        for (name, state, input, at, after, steps, wake) in cases {
            let (next, out) = state.step(input, Stamp(at), &EVERY, &());
            assert_eq!((next, out), (after, steps), "{name}");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}");
        }
    }
}
