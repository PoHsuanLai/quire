//! A timer that runs for exactly as long as an animation takes to settle, started from an event
//! handler (design/05-MOTION.md section 7, the Blitz risk table: "timers start in handlers").
//!
//! The settle task belongs to the hook's owner and is dropped with it: a palette unmounted
//! before its entrance settles takes its timer with it (sill FINDINGS Q45, `crate::task`).

use crate::components::vocab::StaggerIndex;
use crate::motion::anim::Anim;
use crate::motion::settle::settle;
use crate::root::env::{Env, use_env_signal};
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;

/// Where a motion timer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimerPhase {
    /// Not started.
    #[default]
    Idle,
    /// Started; the animation is playing.
    Running,
    /// The animation has settled.
    Settled,
}

/// A settle timer for one animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionTimer {
    anim: Anim,
    phase: Signal<TimerPhase>,
    env: Signal<Env>,
    task: Signal<Option<Task>>,
    scope: ScopeId,
}

impl MotionTimer {
    /// Start (or restart) the timer; `on_settled` runs once, at `settle(anim, level, 0)`. A
    /// timer whose owner has unmounted does nothing, and one running when its owner unmounts is
    /// dropped with it: `on_settled` never runs for a component that is gone.
    pub fn start(&self, on_settled: EventHandler<()>) {
        let _ = self.try_start(on_settled);
    }

    fn try_start(&self, on_settled: EventHandler<()>) -> Result<(), Gone> {
        let level = try_get(self.env)?.resolved.motion;
        let length = settle(self.anim, level, StaggerIndex::default());
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.phase, TimerPhase::Running)?;
        let (phase, task) = (self.phase, self.task);
        let started = spawn_in(self.scope, async move {
            sleep(length).await;
            if settled(phase, task).is_ok() {
                on_settled.call(());
            }
        });
        try_set(self.task, Some(started))
    }

    /// Stop a running timer without settling it: `on_settled` never runs, and the timer is
    /// idle again. For a motion taken back before it ends (an OSD shown again while it fades out,
    /// sill FINDINGS Q76). A timer that is not running, or whose owner is gone, is left as it is.
    pub fn cancel(&self) {
        let _ = self.try_cancel();
    }

    fn try_cancel(&self) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
            try_set(self.task, None)?;
            try_set(self.phase, TimerPhase::Idle)?;
        }
        Ok(())
    }

    /// Where the timer is.
    pub fn phase(&self) -> TimerPhase {
        self.phase
            .try_read()
            .map_or(TimerPhase::Settled, |phase| *phase)
    }
}

/// The timer's end: settled, and no task running.
fn settled(phase: Signal<TimerPhase>, task: Signal<Option<Task>>) -> Result<(), Gone> {
    try_set(phase, TimerPhase::Settled)?;
    try_set(task, None)
}

/// A settle timer for `anim`, reading the motion level from the enclosing `Ds`.
pub fn use_motion_timer(anim: Anim) -> MotionTimer {
    MotionTimer {
        anim,
        phase: use_signal(TimerPhase::default),
        env: use_env_signal(),
        task: use_signal(|| None),
        scope: use_hook(current_scope_id),
    }
}
