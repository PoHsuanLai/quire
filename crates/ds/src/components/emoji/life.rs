//! Playing a [`script`](super::script) (design/25-EMOJI.md section 5). A wake (mounting, a new
//! [`WakeStamp`], a new mood, a new pick) cancels the running script and starts the next; the
//! task that plays it ends on a rest frame, so once the awake window has closed nothing is
//! scheduled and nothing is painted (the idle-frame rule). A task owned by the component's
//! scope, dropped with it.

use super::disc::EmojiPlayback;
use super::id::EmojiId;
use super::script::{MoodChange, Playing, Shown, Step, resting, script};
use crate::appearance::MotionLevel;
use crate::components::user_picture::{Mood, WakeStamp};
use crate::root::env::{Env, use_env_signal};
use crate::task::{spawn_in, try_get, try_set};
use crate::time::sleep;
use crate::tokens::DurationToken;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;

/// What a wake is keyed on.
type Seen = (EmojiId, Mood, WakeStamp, EmojiPlayback);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Player {
    shown: Signal<Shown>,
    task: Signal<Option<Task>>,
    env: Signal<Env>,
    scope: ScopeId,
}

/// The frame to draw now for `user` in `mood`, running a new script on every wake.
pub(crate) fn use_frames(
    user: EmojiId,
    mood: Mood,
    wake: WakeStamp,
    playback: EmojiPlayback,
) -> Shown {
    let player = Player {
        shown: use_signal(|| resting(user, mood)),
        task: use_signal(|| None),
        env: use_env_signal(),
        scope: use_hook(current_scope_id),
    };
    // The last wake seen lives in a plain value, not a signal written while rendering (as
    // `use_bump_on`).
    let mut seen = use_hook(|| CopyValue::new(None::<Seen>));
    let before = *seen.peek();
    if before != Some((user, mood, wake, playback)) {
        seen.set(Some((user, mood, wake, playback)));
        let change = match before {
            Some((_, was, _, _)) if was != mood => MoodChange::Changed,
            _ => MoodChange::Same,
        };
        queue_effect(move || woken(player, user, mood, change, playback));
    }
    *player.shown.read()
}

fn woken(player: Player, user: EmojiId, mood: Mood, change: MoodChange, playback: EmojiPlayback) {
    if let Ok(Some(running)) = try_get(player.task) {
        running.cancel();
    }
    let Ok(env) = try_get(player.env) else {
        return;
    };
    let level = env.resolved.motion;
    let playing = match (level, playback) {
        (MotionLevel::Reduced, _) | (_, EmojiPlayback::Still) => Playing::Stills,
        (MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra, EmojiPlayback::Awake) => {
            Playing::Frames
        }
    };
    let window = DurationToken::Awake.duration(MotionLevel::Standard);
    let steps = script(user, mood, change, playing, window);
    let shown = player.shown;
    let task = spawn_in(player.scope, async move {
        for step in steps {
            match step {
                Step::Show(frame) => {
                    if try_set(shown, frame).is_err() {
                        return;
                    }
                }
                Step::Wait(hold) => sleep(hold).await,
            }
        }
    });
    let mut slot = player.task;
    if let Ok(mut running) = slot.try_write() {
        *running = Some(task);
    }
}
