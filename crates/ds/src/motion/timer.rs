//! A timer that runs for exactly as long as an animation takes to settle, started from an event
//! handler (design/05-MOTION.md section 7, the Blitz risk table: "timers start in handlers").
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::motion::anim::Anim;
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
}

impl MotionTimer {
    /// Start (or restart) the timer; `on_settled` runs once, at `settle(anim, level, 0)`.
    pub fn start(&self, on_settled: EventHandler<()>) {
        todo!()
    }

    /// Where the timer is.
    pub fn phase(&self) -> TimerPhase {
        todo!()
    }
}

/// A settle timer for `anim`, reading the motion level from the enclosing `Ds`.
pub fn use_motion_timer(anim: Anim) -> MotionTimer {
    todo!()
}
