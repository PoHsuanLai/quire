//! The frame timer under the orb's turn: it runs only while the orb is active and motion is not
//! reduced, and costs nothing otherwise.

use super::model::Turn;
use super::step::turn_after;
use crate::motion::detail::level::use_level;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;
use ds_core::time::FRAME_TICK;
use ds_core::time::clock::{now, since, sleep};
use ds_core::vocab::Activity;
use ds_style::appearance::motion::MotionLevel;
use ds_style::task::{Gone, spawn_in, try_get, try_set};
use std::time::Duration;

/// Where the glows are now. While `activity` is `Active` (and motion is not `Reduced`) they turn
/// once per `period`, a frame every [`FRAME_TICK`]; the moment it is `Inactive` the timer is
/// cancelled and they hold where they stand, so an idle orb wakes nothing. Under `Reduced` they
/// stand at the start.
pub(crate) fn use_turn(activity: Activity, period: Duration) -> Turn {
    let level = use_level();
    let clock = Clock {
        turn: use_signal(Turn::default),
        task: use_signal(|| None),
        scope: use_hook(current_scope_id),
    };
    let moving = matches!(activity, Activity::Active) && level.now() != MotionLevel::Reduced;
    let want = moving.then_some(period);
    let mut seen = use_hook(|| CopyValue::new(None::<Duration>));
    if *seen.peek() != want {
        seen.set(want);
        queue_effect(move || {
            let _ = clock.follow(want);
        });
    }
    match level.now() {
        MotionLevel::Reduced => Turn::default(),
        _ => (clock.turn)(),
    }
}

/// The turn and the timer that advances it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Clock {
    turn: Signal<Turn>,
    task: Signal<Option<Task>>,
    scope: ScopeId,
}

impl Clock {
    /// Stop the old timer and, when `period` is asked for, start turning from where the glows
    /// stand.
    fn follow(self, period: Option<Duration>) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)?;
        let Some(period) = period else {
            return Ok(());
        };
        let from = try_get(self.turn)?;
        let started = now();
        let task = spawn_in(self.scope, async move {
            let _ = self.run(from, started, period).await;
        });
        try_set(self.task, Some(task))
    }

    /// Set each frame's turn, for as long as the task lives.
    async fn run(
        self,
        from: Turn,
        started: std::time::Instant,
        period: Duration,
    ) -> Result<(), Gone> {
        loop {
            sleep(FRAME_TICK).await;
            try_set(self.turn, turn_after(from, since(started), period))?;
        }
    }
}
