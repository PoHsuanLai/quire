//! A sheet's life when its host says whether it is shown (sill FINDINGS Q90): it enters with
//! `peek-in`, and when hidden it leaves with `sheet-out` and reports `on_hidden` once that has
//! settled, so a host that unmaps its surface never cuts the exit short. Shown again while
//! leaving, it enters again and `on_hidden` never runs.
//!
//! The stage lives in a `CopyValue`, not a signal: it changes in render, from the caller's
//! `shown`, and the two settle timers' own phases are what re-render the sheet.

use crate::components::tooltip::Shown;
use crate::motion::anim::Anim;
use crate::motion::timer::{MotionTimer, TimerPhase, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Where a sheet is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Shown: entering until its entrance settles, then at rest.
    Up,
    /// Hidden, playing its exit.
    Leaving,
    /// Hidden, and its exit has settled (or it mounted hidden): nothing drawn.
    Gone,
}

/// What one render's `shown` does to a sheet at `stage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Nothing changes.
    Stay,
    /// Shown after leaving or gone: play the entrance again and rejoin the layer stack.
    Enter,
    /// Hidden while up: play the exit and leave the layer stack.
    Leave,
}

/// The step from `stage` given the caller's `shown`.
pub(crate) fn step(stage: Stage, shown: Shown) -> Step {
    match (stage, shown) {
        (Stage::Leaving | Stage::Gone, Shown::Visible) => Step::Enter,
        (Stage::Up, Shown::Hidden) => Step::Leave,
        (Stage::Up, Shown::Visible) | (Stage::Leaving | Stage::Gone, Shown::Hidden) => Step::Stay,
    }
}

/// The sheet's showing for one render.
#[derive(Clone, Copy)]
pub(crate) struct SheetShowing {
    stage: CopyValue<Stage>,
    entrance: MotionTimer,
    exit: MotionTimer,
    /// What this render's `shown` changed, for the caller to act on the layer stack.
    pub(crate) step: Step,
}

impl SheetShowing {
    /// Whether the sheet is drawn at all: not once gone. Reads the exit timer's phase so the
    /// sheet re-renders when its exit settles (the stage itself is not a signal); the stage alone
    /// decides, since a new exit's timer still reads settled until its effect restarts it.
    pub(crate) fn drawn(&self) -> bool {
        let _subscribed = self.exit.phase();
        *self.stage.peek() != Stage::Gone
    }

    /// `data-presence`: entering, present, or leaving.
    pub(crate) fn slug(&self) -> &'static str {
        match (*self.stage.peek(), self.entrance.phase()) {
            (Stage::Up, TimerPhase::Settled) => "present",
            (Stage::Up, TimerPhase::Idle | TimerPhase::Running) => "entering",
            (Stage::Leaving | Stage::Gone, _) => "leaving",
        }
    }

    /// Whether the sheet is on its way out: its scrim fades with it.
    pub(crate) fn leaving(&self) -> bool {
        *self.stage.peek() == Stage::Leaving
    }
}

/// The showing hook. `shown` is the caller's (`None`: always shown, as a sheet was before);
/// `on_hidden` runs when a hidden sheet's exit settles.
pub(crate) fn use_sheet_showing(
    shown: Option<Shown>,
    on_hidden: Option<EventHandler<()>>,
) -> SheetShowing {
    let now = shown.unwrap_or(Shown::Visible);
    let entrance = use_motion_timer(Anim::PeekIn);
    let exit = use_motion_timer(Anim::SheetOut);
    let mut stage = use_hook(|| {
        CopyValue::new(match now {
            Shown::Visible => Stage::Up,
            Shown::Hidden => Stage::Gone,
        })
    });
    use_hook(|| {
        if now == Shown::Visible {
            entrance.start(EventHandler::new(|()| {}));
        }
    });
    // Both handlers are made in render: an effect has no scope to make a handler in.
    let settled = EventHandler::new(move |()| {
        if *stage.peek() == Stage::Leaving {
            stage.set(Stage::Gone);
            if let Some(on_hidden) = on_hidden {
                on_hidden.call(());
            }
        }
    });
    let entered = EventHandler::new(|()| {});
    let step = step(*stage.peek(), now);
    match step {
        Step::Stay => {}
        Step::Enter => {
            stage.set(Stage::Up);
            queue_effect(move || entrance.start(entered));
        }
        Step::Leave => {
            stage.set(Stage::Leaving);
            queue_effect(move || exit.start(settled));
        }
    }
    SheetShowing {
        stage,
        entrance,
        exit,
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
