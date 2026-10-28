//! The caret's blink. Blitz paints a focused field's caret on every frame it draws and has no
//! blink of its own, so the field keeps the phase: `data-blink=off` on its wrap makes the
//! renderer's caret (and a secure field's drawn one) transparent for the off half.
//!
//! The caret shows at once whenever it arrives, moves or is typed at, so it never vanishes under
//! the key just pressed, and blinks [`BLINK_ON`] on, [`BLINK_OFF`] off after that. It is bounded
//! (design/26 R3): after [`BLINK_FOR`] with nothing typed it holds still, shown, and the field
//! paints no more frames until the caret next moves, as GNOME's `cursor-blink-timeout` does.
//! Under Reduced motion it never blinks.

use dioxus::core::{ScopeId, Task, current_scope_id};
use dioxus::prelude::*;
use ds_core::time::clock::sleep;
use ds_motion::detail::level::{Level, use_level};
use ds_style::appearance::motion::MotionLevel;
use ds_style::task::{spawn_in, try_set_if_changed};
use std::time::Duration;

/// How long the caret shows in each blink.
pub(crate) const BLINK_ON: Duration = Duration::from_millis(500);
/// How long the caret is hidden in each blink.
pub(crate) const BLINK_OFF: Duration = Duration::from_millis(500);
/// How long the caret blinks after it last moved before it holds still.
pub(crate) const BLINK_FOR: Duration = Duration::from_secs(10);

/// A field's blink: its phase and the task that toggles it.
#[derive(Clone, Copy)]
pub(crate) struct CaretBlink {
    /// Whether the caret shows: `false` for the off half. (Also `true` when the field has no
    /// caret or the blink has stopped.)
    lit: Signal<bool>,
    task: CopyValue<Option<Task>>,
    level: Level,
    owner: ScopeId,
}

impl CaretBlink {
    /// The hook: one per field, kept across renders.
    pub(crate) fn use_new() -> Self {
        CaretBlink {
            lit: use_signal(|| true),
            task: use_hook(|| CopyValue::new(None)),
            level: use_level(),
            owner: use_hook(current_scope_id),
        }
    }

    /// The wrap's `data-blink`: `off` in the hidden half, absent otherwise.
    pub(crate) fn attr(&self) -> Option<&'static str> {
        (!(self.lit)()).then_some("off")
    }

    /// The caret arrived, moved or was typed at: it shows, and blinks again from the start.
    pub(crate) fn restart(&self) {
        self.halt();
        if self.level.now() == MotionLevel::Reduced {
            return;
        }
        let lit = self.lit;
        let task = spawn_in(self.owner, async move {
            let blinks = BLINK_FOR.as_millis() / (BLINK_ON + BLINK_OFF).as_millis();
            for _ in 0..blinks {
                sleep(BLINK_ON).await;
                if try_set_if_changed(lit, false).is_err() {
                    return;
                }
                sleep(BLINK_OFF).await;
                if try_set_if_changed(lit, true).is_err() {
                    return;
                }
            }
        });
        let mut slot = self.task;
        slot.set(Some(task));
    }

    /// The caret left: no blink until it comes back.
    pub(crate) fn stop(&self) {
        self.halt();
    }

    fn halt(&self) {
        let mut slot = self.task;
        if let Some(running) = slot.write().take() {
            running.cancel();
        }
        let _ = try_set_if_changed(self.lit, true);
    }
}
