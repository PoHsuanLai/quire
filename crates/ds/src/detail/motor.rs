//! The frame driver under every Rust-played detail: a glide advanced by a timer that asks for a
//! frame every [`FRAME`] while it moves and stops when it lands (R3), retargeting from where it is
//! (R10).

use super::glide::{FRAME, Glide, Pose};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::Instant;

/// A glide and the timer that advances it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Motor {
    pose: Signal<Pose>,
    task: Signal<Option<Task>>,
    scope: ScopeId,
}

impl Motor {
    /// Where it is, subscribing the caller's render to its frames.
    pub(crate) fn pose(self) -> Pose {
        (self.pose)()
    }

    /// Where it is, without subscribing.
    pub(crate) fn peek(self) -> Pose {
        self.pose.try_peek().map_or(Pose::default(), |pose| *pose)
    }

    /// Play `glide` from its start; a glide already playing stops where it is. Call from an
    /// effect or a handler.
    pub(crate) fn play(self, glide: Glide) {
        let _ = self.try_play(glide);
    }

    /// Stand at `at` at once, with no frames.
    pub(crate) fn snap(self, at: i64) {
        self.play(Glide::still(at));
    }

    fn try_play(self, glide: Glide) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)?;
        let started = Instant::now();
        let run = try_get(self.pose)?.run.wrapping_add(1);
        let posed = move |elapsed| Pose {
            run,
            ..glide.at(elapsed)
        };
        try_set(self.pose, posed(std::time::Duration::ZERO))?;
        if glide.done(std::time::Duration::ZERO) {
            return Ok(());
        }
        let task = spawn_in(self.scope, async move {
            let _ = self.run(glide, started, posed).await;
        });
        try_set(self.task, Some(task))
    }

    async fn run(
        self,
        glide: Glide,
        started: Instant,
        posed: impl Fn(std::time::Duration) -> Pose,
    ) -> Result<(), Gone> {
        loop {
            sleep(FRAME).await;
            let elapsed = started.elapsed();
            try_set(self.pose, posed(elapsed))?;
            if glide.done(elapsed) {
                return try_set(self.task, None);
            }
        }
    }
}

/// A motor standing at `at`.
pub(crate) fn use_motor(at: i64) -> Motor {
    Motor {
        pose: use_signal(|| Glide::still(at).at(std::time::Duration::ZERO)),
        task: use_signal(|| None),
        scope: use_hook(current_scope_id),
    }
}
