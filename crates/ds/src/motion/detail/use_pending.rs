//! The bounded pending loop as a hook (design/26-DETAILS.md R4): a Rust step timer that runs only
//! while an operation's token is live, from its grace to its deadline, then stops.

use super::level::use_level;
use super::operation::Operation;
use super::pending::{PendingFrame, PendingSpec};
use crate::motion::timeline::pending::Pending;
use crate::motion::timeline::playback::{Playback, use_playback};
use crate::style::appearance::motion::MotionLevel;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// What the loop last followed: the operation, the motion level and the frame it drew.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Followed {
    op: Operation,
    level: MotionLevel,
    frame: PendingFrame,
}

/// The frame `op`'s loop shows now. `Idle` until the operation has run for `PendingGrace`, then a
/// step every `--t-pending-step`, then `Stalled` from the token's deadline (or at once after the
/// grace under Reduced) for as long as the operation still runs; `Idle` again the moment it ends.
/// The timer stops at the deadline, so a stuck operation costs 0 frames. `spec` is the look the
/// caller draws the frame with ([`PendingFrame::lit`]); the timing does not depend on it.
///
/// Each run is one frame long: it ends at the instant the next frame is due, and the render that
/// frame causes starts the run from there, at the operation's own age.
pub fn use_pending(op: Operation, spec: PendingSpec) -> PendingFrame {
    let _ = spec;
    let env = use_level();
    let playback = use_playback(Pending::Idle);
    let frame = playback.frame();
    let mut followed = use_hook(|| {
        CopyValue::new(Followed {
            op: Operation::Idle,
            level: MotionLevel::Standard,
            frame: PendingFrame::Idle,
        })
    });
    let now = Followed {
        op,
        level: env.now(),
        frame,
    };
    if *followed.peek() != now {
        followed.set(now);
        queue_effect(move || follow(playback, op, now.level));
    }
    match op {
        Operation::Idle => PendingFrame::Idle,
        Operation::Running(_) => frame,
    }
}

/// Play the run that follows `op`'s loop from its age now.
fn follow(playback: Playback<Pending>, op: Operation, level: MotionLevel) {
    playback.play(match op {
        Operation::Idle => Pending::Idle,
        Operation::Running(token) => Pending::Running {
            deadline: token.deadline(),
            level,
            from: token.elapsed(),
        },
    });
}
