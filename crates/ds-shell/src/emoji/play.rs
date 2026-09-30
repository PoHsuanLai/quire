//! Playing an animated emoji's own animation (design/30 section 2.10, design/25-EMOJI.md): the
//! sheet's frames once through at the frame durations the asset carries, on mounting, on each new
//! [`WakeStamp`] and on each new pick, then at rest on frame 0. Nothing loops and nothing runs
//! between plays (the idle-frame rule). Under Reduced motion, or with
//! [`EmojiPlayback::Still`], only the rest frame is shown. The task that plays it is owned by
//! the component's scope and dropped with it.

use super::disc::EmojiPlayback;
use super::id::EmojiId;
use super::sheet::durations;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;
use ds_core::time::clock::sleep;
use ds_motion::wake::WakeStamp;
use ds_style::appearance::motion::MotionLevel;
use ds_style::scope::{Scope, use_scope_signal};
use ds_style::task::{spawn_in, try_get, try_set};
use std::time::Duration;

/// What a play is keyed on.
type Seen = (EmojiId, WakeStamp, EmojiPlayback);

/// One frame of the animation and how long it is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Held {
    pub(crate) frame: u16,
    pub(crate) hold: Duration,
}

/// The frames of `emoji`'s animation in order, each with the duration the asset gives it.
pub(crate) fn animation(emoji: EmojiId) -> Vec<Held> {
    durations(emoji)
        .into_iter()
        .zip(0u16..)
        .map(|(hold, frame)| Held { frame, hold })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Player {
    frame: Signal<u16>,
    task: Signal<Option<Task>>,
    env: Signal<Scope>,
    scope: ScopeId,
}

/// The frame to draw now for `emoji`: the animation plays through on mounting and on every new
/// `wake` or pick, and ends on the rest frame.
pub(crate) fn use_frame(emoji: EmojiId, wake: WakeStamp, playback: EmojiPlayback) -> u16 {
    let player = Player {
        frame: use_signal(|| 0),
        task: use_signal(|| None),
        env: use_scope_signal(),
        scope: use_hook(current_scope_id),
    };
    // The last play's key lives in a plain value, not a signal written while rendering.
    let mut seen = use_hook(|| CopyValue::new(None::<Seen>));
    let key = (emoji, wake, playback);
    if *seen.peek() != Some(key) {
        seen.set(Some(key));
        queue_effect(move || start(player, emoji, playback));
    }
    *player.frame.read()
}

fn start(player: Player, emoji: EmojiId, playback: EmojiPlayback) {
    if let Ok(Some(running)) = try_get(player.task) {
        running.cancel();
    }
    let _ = try_set(player.frame, 0);
    let Ok(env) = try_get(player.env) else {
        return;
    };
    let frames = match (env.resolved.motion, playback) {
        (MotionLevel::Reduced, _) | (_, EmojiPlayback::Still) => Vec::new(),
        (MotionLevel::Standard, EmojiPlayback::Once) => animation(emoji),
    };
    let frame = player.frame;
    let task = spawn_in(player.scope, async move {
        for held in frames {
            if try_set(frame, held.frame).is_err() {
                return;
            }
            sleep(held.hold).await;
        }
        let _ = try_set(frame, 0);
    });
    let mut slot = player.task;
    if let Ok(mut running) = slot.try_write() {
        *running = Some(task);
    }
}
