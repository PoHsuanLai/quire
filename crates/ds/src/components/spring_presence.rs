//! A surface's life when its host says whether it is shown, for the
//! sheet and the edge panel. Its first showing on mount is an arrival no hand touched, so it
//! plays its entrance keyframe (`peek-in`, `panel-in`). Every change after that is driven motion
//! (design/05 section 14): one spring in Rust, `--present-p` from 0 (gone) to 1
//! (shown), carries the exit and any later entrance, so a show while it leaves turns it back from
//! where it is at the speed it has instead of starting over, and `on_hidden` runs once the spring
//! has come to rest at 0, so a host that unmaps its surface never cuts the exit short.
//!
//! The stage lives in a `CopyValue`, not a signal: it changes in render, from the caller's
//! `shown`, and the entrance timer's phase and the spring's frames are what re-render the
//! surface.

use crate::components::tooltip::Shown;
use crate::detail::Touch;
use crate::motion::anim::Anim;
use crate::motion::timer::{MotionTimer, TimerPhase, use_motion_timer};
use crate::motion::{
    PxPerUnit, SpringMotion, SpringPhase, SpringResponse, SpringSpec, use_spring_motion,
};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// How many pixels the whole showing spans, for the spring's rest: a hundredth of it is well
/// under a visible step of a slide or a fade.
const SHOWING_PX: f32 = 40.0;

/// Where a surface is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Shown: entering until its entrance settles, then at rest.
    Up,
    /// Hidden, on its way out.
    Leaving,
    /// Hidden, and its exit has come to rest (or it mounted hidden): nothing drawn.
    Gone,
}

/// What one render's `shown` does to a surface at `stage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Nothing changes.
    Stay,
    /// Shown after leaving or gone: spring in again (and rejoin the layer stack).
    Enter,
    /// Hidden while up: spring out (and leave the layer stack).
    Leave,
}

/// What moves the surface now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Drive {
    /// Its first entrance, the keyframe in its stylesheet.
    Keyframe,
    /// The spring, from its first hide on.
    Spring,
}

/// The step from `stage` given the caller's `shown`.
pub(crate) fn step(stage: Stage, shown: Shown) -> Step {
    match (stage, shown) {
        (Stage::Leaving | Stage::Gone, Shown::Visible) => Step::Enter,
        (Stage::Up, Shown::Hidden) => Step::Leave,
        (Stage::Up, Shown::Visible) | (Stage::Leaving | Stage::Gone, Shown::Hidden) => Step::Stay,
    }
}

/// Where the spring stands for `shown`.
fn place(shown: Shown) -> f32 {
    match shown {
        Shown::Visible => 1.0,
        Shown::Hidden => 0.0,
    }
}

/// The surface's showing for one render.
#[derive(Clone, Copy)]
pub(crate) struct SpringPresence {
    stage: CopyValue<Stage>,
    drive: CopyValue<Drive>,
    entrance: MotionTimer,
    spring: SpringMotion,
    /// What this render's `shown` changed, for the caller to act on the layer stack.
    pub(crate) step: Step,
}

impl SpringPresence {
    /// Whether the surface is drawn at all: not once gone.
    pub(crate) fn drawn(&self) -> bool {
        *self.stage.peek() != Stage::Gone
    }

    /// `data-presence`: entering, present, or leaving.
    pub(crate) fn slug(&self) -> &'static str {
        match (*self.stage.peek(), *self.drive.peek()) {
            (Stage::Up, Drive::Keyframe) => match self.entrance.phase() {
                TimerPhase::Settled => "present",
                TimerPhase::Idle | TimerPhase::Running => "entering",
            },
            (Stage::Up, Drive::Spring) => {
                let frame = self.spring.peek();
                match (frame.phase(), frame.position() == 1.0) {
                    (SpringPhase::Rest, true) => "present",
                    (SpringPhase::Rest, false) | (SpringPhase::Moving, _) => "entering",
                }
            }
            (Stage::Leaving | Stage::Gone, _) => "leaving",
        }
    }

    /// Whether the surface is on its way out: its scrim fades with it.
    pub(crate) fn leaving(&self) -> bool {
        *self.stage.peek() == Stage::Leaving
    }

    /// `data-drive`: `spring` once the spring draws it, nothing while its first entrance plays.
    pub(crate) fn drive(&self) -> Option<&'static str> {
        match *self.drive.peek() {
            Drive::Keyframe => None,
            Drive::Spring => Some("spring"),
        }
    }

    /// The spring's place for the stylesheet, `--present-p:0.500`.
    pub(crate) fn style(&self) -> String {
        format!("--present-p:{}", self.spring.peek().css())
    }
}

/// The showing hook. `shown` is the caller's (`None`: always shown); `entrance` is the keyframe
/// its first showing plays; `on_hidden` runs when a hidden surface's spring comes to rest.
pub(crate) fn use_spring_presence(
    shown: Option<Shown>,
    on_hidden: Option<EventHandler<()>>,
    entrance: Anim,
) -> SpringPresence {
    let now = shown.unwrap_or(Shown::Visible);
    let timer = use_motion_timer(entrance);
    let spring = use_spring_motion(place(now), PxPerUnit(SHOWING_PX));
    // Subscribe: each frame, and the rest, renders the surface again.
    let frame = spring.frame();
    let mut stage = use_hook(|| {
        CopyValue::new(match now {
            Shown::Visible => Stage::Up,
            Shown::Hidden => Stage::Gone,
        })
    });
    let mut drive = use_hook(|| CopyValue::new(Drive::Keyframe));
    use_hook(|| {
        if now == Shown::Visible {
            timer.start(EventHandler::new(|()| {}));
        }
    });
    let step = step(*stage.peek(), now);
    match step {
        Step::Stay => {}
        Step::Enter => {
            stage.set(Stage::Up);
            drive.set(Drive::Spring);
            let spec = SpringSpec::for_touch(Touch::Remote);
            queue_effect(move || spring.go(1.0, spec));
        }
        Step::Leave => {
            stage.set(Stage::Leaving);
            drive.set(Drive::Spring);
            let spec = SpringSpec::for_touch(Touch::Remote).response(SpringResponse::Quick);
            queue_effect(move || spring.go(0.0, spec));
        }
    }
    let out = *stage.peek() == Stage::Leaving && step == Step::Stay;
    if out && frame.phase() == SpringPhase::Rest && frame.position() == 0.0 {
        stage.set(Stage::Gone);
        if let Some(on_hidden) = on_hidden {
            queue_effect(move || on_hidden.call(()));
        }
    }
    SpringPresence {
        stage,
        drive,
        entrance: timer,
        spring,
        step,
    }
}

#[cfg(test)]
mod tests {
    use super::{Stage, Step, step};
    use crate::components::tooltip::Shown;

    #[test]
    fn each_stage_steps_by_what_the_host_asks() {
        const CASES: &[(Stage, Shown, Step)] = &[
            (Stage::Up, Shown::Visible, Step::Stay),
            (Stage::Up, Shown::Hidden, Step::Leave),
            (Stage::Leaving, Shown::Hidden, Step::Stay),
            (Stage::Leaving, Shown::Visible, Step::Enter),
            (Stage::Gone, Shown::Hidden, Step::Stay),
            (Stage::Gone, Shown::Visible, Step::Enter),
        ];
        for &(stage, shown, want) in CASES {
            assert_eq!(step(stage, shown), want, "{stage:?} {shown:?}");
        }
    }
}
