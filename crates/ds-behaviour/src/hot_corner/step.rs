//! The corner's transition.
//!
//! | State | Input | Next | Out |
//! |---|---|---|---|
//! | Armed | Enter at t | Dwelling { until: t + dwell } | none |
//! | Dwelling | Leave | Armed | none |
//! | Dwelling { until } | Elapsed at t >= until | Spent { In, Running { until: t + rearm } } | Fire |
//! | Spent { rearm } | Enter | Spent { In, rearm } | none |
//! | Spent { Over } | Leave | Armed | none |
//! | Spent { Running } | Leave | Spent { Out, Running } | none |
//! | Spent { pointer, Running { until } } | Elapsed at t >= until | Armed if the pointer is out, else Spent { In, Over } | none |
//!
//! Every other pair (an early `Elapsed`, a second `Enter` while dwelling) changes nothing. The
//! machine is idle in `Armed`, in `Spent` once the re-arm is over, and runs no timer there.

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::model::{Corner, CornerIn, CornerOut, CornerParams, Inside, Rearm};
use crate::span;

/// What a step wants done, beside the state it leaves.
type Step = (Corner, Vec<CornerOut>);

impl Machine for Corner {
    type In = CornerIn;
    type Out = CornerOut;
    type Params = CornerParams;
    type Ctx = ();

    fn step(self, input: CornerIn, at: Stamp, params: &CornerParams, _: &()) -> Step {
        match (self, input) {
            (Corner::Armed, CornerIn::Enter) => quiet(Corner::Dwelling {
                until: span::after(at, params.dwell),
            }),
            (Corner::Dwelling { .. }, CornerIn::Leave) => quiet(Corner::Armed),
            (Corner::Dwelling { until }, CornerIn::Elapsed) if at >= until => (
                Corner::Spent {
                    pointer: Inside::In,
                    rearm: Rearm::Running {
                        until: span::after(at, params.rearm),
                    },
                },
                vec![CornerOut::Fire],
            ),
            (Corner::Spent { rearm, .. }, CornerIn::Enter) => spent(Inside::In, rearm),
            (
                Corner::Spent {
                    rearm: Rearm::Over, ..
                },
                CornerIn::Leave,
            ) => quiet(Corner::Armed),
            (Corner::Spent { rearm, .. }, CornerIn::Leave) => spent(Inside::Out, rearm),
            (
                Corner::Spent {
                    pointer,
                    rearm: Rearm::Running { until },
                },
                CornerIn::Elapsed,
            ) if at >= until => rearm_ran_out(pointer),
            (state, _) => quiet(state),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Corner::Dwelling { until }
            | Corner::Spent {
                rearm: Rearm::Running { until },
                ..
            } => Some(*until),
            Corner::Armed
            | Corner::Spent {
                rearm: Rearm::Over, ..
            } => None,
        }
    }
}

/// `state`, asking for nothing.
fn quiet(state: Corner) -> Step {
    (state, Vec::new())
}

/// Spent with the pointer at `pointer` and the re-arm in `rearm`.
fn spent(pointer: Inside, rearm: Rearm) -> Step {
    quiet(Corner::Spent { pointer, rearm })
}

/// The re-arm ran out: armed if the pointer has left, else waiting for it to.
fn rearm_ran_out(pointer: Inside) -> Step {
    match pointer {
        Inside::Out => quiet(Corner::Armed),
        Inside::In => spent(Inside::In, Rearm::Over),
    }
}
