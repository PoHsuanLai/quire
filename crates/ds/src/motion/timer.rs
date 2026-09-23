//! A timer that runs for exactly as long as an animation takes to settle, started from an event
//! handler (design/05-MOTION.md section 7, the Blitz risk table: "timers start in handlers").

use crate::components::vocab::StaggerIndex;
use crate::motion::anim::Anim;
use crate::motion::settle::settle;
use crate::root::env::{Env, use_env_signal};
use crate::time::{sleep, spawn_in};
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
    /// Start (or restart) the timer; `on_settled` runs once, at `settle(anim, level, 0)`.
    pub fn start(&self, on_settled: EventHandler<()>) {
        let length = settle(
            self.anim,
            self.env.peek().resolved.motion,
            StaggerIndex::default(),
        );
        let mut phase = self.phase;
        let mut task = self.task;
        if let Some(running) = *task.peek() {
            running.cancel();
        }
        phase.set(TimerPhase::Running);
        let started = spawn_in(self.scope, async move {
            sleep(length).await;
            phase.set(TimerPhase::Settled);
            task.set(None);
            on_settled.call(());
        });
        task.set(Some(started));
    }

    /// Where the timer is.
    pub fn phase(&self) -> TimerPhase {
        (self.phase)()
    }
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
