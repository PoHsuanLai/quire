//! The pending loop as a hook (design/26-DETAILS.md R4): a Rust step timer that runs only while
//! an operation's token is live, and stops the moment it ends.

use super::operation::Operation;
use super::pending::{PendingFrame, PendingSpec};
use crate::motion::timeline::pending::Pending;
use crate::motion::timeline::playback::{Playback, use_playback};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The frame `op`'s loop shows now: `Idle` with no operation, else a step every
/// `--t-spin-step` from the first frame on, for as long as the operation runs; `Idle` again the
/// moment it ends. `spec` is the look the caller draws the frame with ([`PendingFrame::lit`]);
/// the timing does not depend on it.
///
/// Each run is one frame long: it ends at the instant the next frame is due, and the render that
/// frame causes starts the run from there, at the operation's own age.
pub fn use_pending(op: Operation, spec: PendingSpec) -> PendingFrame {
    let _ = spec;
    let playback = use_playback(Pending::Idle);
    let frame = playback.frame();
    let mut followed = use_hook(|| CopyValue::new((Operation::Idle, PendingFrame::Idle)));
    let now = (op, frame);
    if *followed.peek() != now {
        followed.set(now);
        queue_effect(move || follow(playback, op));
    }
    match op {
        Operation::Idle => PendingFrame::Idle,
        Operation::Running(_) => frame,
    }
}

/// Play the run that follows `op`'s loop from its age now.
fn follow(playback: Playback<Pending>, op: Operation) {
    playback.play(match op {
        Operation::Idle => Pending::Idle,
        Operation::Running(token) => Pending::Running {
            from: token.elapsed(),
        },
    });
}
