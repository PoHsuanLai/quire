//! The corner's transition.

use super::model::{CornerIn, CornerMachine, CornerOut, CornerParams, Inside, Phase, Rearm, Token};

/// What a step wants done, beside the machine it leaves.
type Step = (CornerMachine, Vec<CornerOut>);

/// `machine` after `input`, and the timers and firing it wants. `Enter` starts a dwell from
/// `Armed`; `Leave` cancels a dwell; a live `DwellElapsed` fires and starts the re-arm; a live
/// `RearmElapsed` re-arms at once if the pointer has left, else the next `Leave` does. A stale
/// token changes nothing.
pub fn step(machine: CornerMachine, input: CornerIn, params: CornerParams) -> Step {
    match (machine.phase, input) {
        (Phase::Armed, CornerIn::Enter) => start_dwell(machine, params),
        (Phase::Dwelling(_), CornerIn::Leave) => to(machine, Phase::Armed),
        (Phase::Dwelling(live), CornerIn::DwellElapsed(token)) if live == token => {
            fire(machine, params)
        }
        (Phase::Spent { rearm, .. }, CornerIn::Enter) => spent(machine, Inside::In, rearm),
        (
            Phase::Spent {
                rearm: Rearm::Over, ..
            },
            CornerIn::Leave,
        ) => to(machine, Phase::Armed),
        (Phase::Spent { rearm, .. }, CornerIn::Leave) => spent(machine, Inside::Out, rearm),
        (
            Phase::Spent {
                pointer,
                rearm: Rearm::Running(live),
            },
            CornerIn::RearmElapsed(token),
        ) if live == token => rearm_ran_out(machine, pointer),
        _ => (machine, Vec::new()),
    }
}

/// The token after `token`.
fn following(token: Token) -> Token {
    Token(token.0.wrapping_add(1))
}

/// `machine` in `phase`, asking for nothing.
fn to(machine: CornerMachine, phase: Phase) -> Step {
    (CornerMachine { phase, ..machine }, Vec::new())
}

/// `machine` spent with the pointer at `pointer` and the re-arm in `rearm`.
fn spent(machine: CornerMachine, pointer: Inside, rearm: Rearm) -> Step {
    to(machine, Phase::Spent { pointer, rearm })
}

/// The dwell begins under the next token.
fn start_dwell(machine: CornerMachine, params: CornerParams) -> Step {
    let token = machine.next;
    (
        CornerMachine {
            phase: Phase::Dwelling(token),
            next: following(token),
        },
        vec![CornerOut::StartDwell(token, params.dwell)],
    )
}

/// The dwell ran out: the corner acts and the re-arm begins under the next token, the pointer
/// still in.
fn fire(machine: CornerMachine, params: CornerParams) -> Step {
    let token = machine.next;
    (
        CornerMachine {
            phase: Phase::Spent {
                pointer: Inside::In,
                rearm: Rearm::Running(token),
            },
            next: following(token),
        },
        vec![CornerOut::Fire, CornerOut::StartRearm(token, params.rearm)],
    )
}

/// The re-arm ran out: armed if the pointer has left, else waiting for it to.
fn rearm_ran_out(machine: CornerMachine, pointer: Inside) -> Step {
    match pointer {
        Inside::Out => to(machine, Phase::Armed),
        Inside::In => spent(machine, Inside::In, Rearm::Over),
    }
}
