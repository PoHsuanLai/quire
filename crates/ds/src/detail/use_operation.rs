//! The operation a component's own Pending moment is, for a component whose state says "busy"
//! but carries no token (design/26-DETAILS.md section 7, D0b: `ModuleState::Busy`,
//! `PromptState::Checking`).

use super::cue::Cue;
use super::moment::Moment;
use super::operation::{Deadline, Operation, PendingToken};
use dioxus::prelude::*;

/// `Operation::Running` for as long as `cue` is a Pending moment, under a token started when that
/// moment arrived with the cap as its deadline; `Operation::Idle` otherwise. A later Pending
/// moment (a new change into a busy state) is a new operation.
pub fn use_operation(cue: Cue) -> Operation {
    let mut minted = use_hook(|| CopyValue::new(None::<(u32, PendingToken)>));
    if cue.moment() != Moment::Pending {
        return Operation::Idle;
    }
    let current = *minted.peek();
    match current {
        Some((serial, token)) if serial == cue.serial() => Operation::Running(token),
        Some(_) | None => {
            let token = PendingToken::start(Deadline::cap());
            minted.set(Some((cue.serial(), token)));
            Operation::Running(token)
        }
    }
}
