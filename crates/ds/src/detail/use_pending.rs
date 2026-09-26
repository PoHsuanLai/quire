//! The bounded pending loop as a hook (design/26-DETAILS.md R4): a Rust step timer that runs only
//! while an operation's token is live, from its grace to its deadline, then stops.

use super::level::{Level, use_level};
use super::operation::{Operation, PendingToken};
use super::pending::{PendingFrame, PendingSpec, frame_at, next_due};
use crate::task::{Gone, spawn_in, try_get, try_set, try_set_if_changed};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;

/// The frame `op`'s loop shows now. `Idle` until the operation has run for `PendingGrace`, then a
/// step every `--t-pending-step`, then `Stalled` from the token's deadline (or at once under
/// Reduced) for as long as the operation still runs; `Idle` again the moment it ends. The timer
/// stops at the deadline, so a stuck operation costs 0 frames. `spec` is the look the caller
/// draws the frame with ([`PendingFrame::lit`]); the timing does not depend on it.
pub fn use_pending(op: Operation, spec: PendingSpec) -> PendingFrame {
    let _ = spec;
    let clock = Clock {
        frame: use_signal(|| PendingFrame::Idle),
        task: use_signal(|| None),
        env: use_level(),
        scope: use_hook(current_scope_id),
    };
    let mut seen = use_hook(|| CopyValue::new(Operation::Idle));
    if *seen.peek() != op {
        seen.set(op);
        queue_effect(move || {
            let _ = clock.follow(op);
        });
    }
    match op {
        Operation::Idle => PendingFrame::Idle,
        Operation::Running(_) => (clock.frame)(),
    }
}

/// The loop's timer and the frame it last set.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Clock {
    frame: Signal<PendingFrame>,
    task: Signal<Option<Task>>,
    env: Level,
    scope: ScopeId,
}

impl Clock {
    /// Stop the old operation's timer and start `op`'s.
    fn follow(self, op: Operation) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)?;
        try_set_if_changed(self.frame, PendingFrame::Idle)?;
        let Operation::Running(token) = op else {
            return Ok(());
        };
        let started = spawn_in(self.scope, async move {
            let _ = self.run(token).await;
        });
        try_set(self.task, Some(started))
    }

    /// Set each frame when it is due, until the loop holds still.
    async fn run(self, token: PendingToken) -> Result<(), Gone> {
        loop {
            let level = self.env.now();
            let elapsed = token.elapsed();
            try_set_if_changed(self.frame, frame_at(elapsed, token.deadline(), level))?;
            match next_due(elapsed, token.deadline(), level) {
                Some(wait) => sleep(wait).await,
                None => return try_set(self.task, None),
            }
        }
    }
}
