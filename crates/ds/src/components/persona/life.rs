//! What makes a persona move (design/24-PERSONA.md section 4), and what makes it stop.
//!
//! Every motion is a pulse fired from an effect or a timer and put back at rest at its
//! `settle`, so nothing loops and nothing stays on the element (the idle-frame rule: a surface
//! at rest paints 0 frames). A wake (mounting, a new [`WakeStamp`], a mood change) opens a
//! 20 s awake window: `persona-breathe` runs once across it in every mood (a sleeping persona
//! breathes too), and in a mood with open eyes a
//! Rust timer blinks at the gaps [`PersonaSpec::blink_gaps`] gives, then stops at the window's
//! end. A mood change also plays that mood's one motion: a wince, a hop or a drifting `z`.
//! Under Reduced motion nothing is fired: moods change instantly and nothing blinks.

use super::mood::{Blinking, Mood, WakeStamp};
use super::spec::PersonaSpec;
use crate::appearance::MotionLevel;
use crate::components::vocab::{PulseKey, StaggerIndex};
use crate::motion::{Anim, MotionTimer, Pulse, TimerPhase, settle, use_motion_timer, use_pulse};
use crate::root::env::{Env, use_env_signal};
use crate::task::{spawn_in, try_get};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;

/// One motion that plays once when told and is at rest otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Play {
    pulse: Pulse,
    timer: MotionTimer,
    settled: EventHandler<()>,
}

impl Play {
    /// Play it from its first frame; at rest again at its settle.
    fn play(self) {
        self.pulse.fire();
        self.timer.start(self.settled);
    }

    /// The key to render: the fired alias while the timer runs, rest otherwise.
    fn key(self) -> PulseKey {
        match self.timer.phase() {
            TimerPhase::Running => self.pulse.key(),
            TimerPhase::Idle | TimerPhase::Settled => PulseKey::rest(self.pulse.key().anim()),
        }
    }
}

fn use_play(anim: Anim) -> Play {
    Play {
        pulse: use_pulse(anim),
        timer: use_motion_timer(anim),
        settled: use_hook(|| EventHandler::new(|()| {})),
    }
}

/// The keys each moving part renders this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Life {
    pub blink: PulseKey,
    pub breathe: PulseKey,
    pub wince: PulseKey,
    pub hop: PulseKey,
    pub drift: PulseKey,
}

/// The persona's plays and its blink timer.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Plays {
    blink: Play,
    breathe: Play,
    wince: Play,
    hop: Play,
    drift: Play,
    blinker: Signal<Option<Task>>,
    env: Signal<Env>,
    scope: ScopeId,
}

/// Run a persona's life: wake on mount, on a new `wake` and on every mood change.
pub(crate) fn use_life(spec: PersonaSpec, mood: Mood, wake: WakeStamp) -> Life {
    let plays = Plays {
        blink: use_play(Anim::PersonaBlink),
        breathe: use_play(Anim::PersonaBreathe),
        wince: use_play(Anim::PersonaWince),
        hop: use_play(Anim::PersonaHop),
        drift: use_play(Anim::PersonaDrift),
        blinker: use_signal(|| None),
        env: use_env_signal(),
        scope: use_hook(current_scope_id),
    };
    // The last mood and stamp seen live in a plain value: a signal written while rendering
    // would schedule a second render for nothing (as `use_bump_on`).
    let mut seen = use_hook(|| CopyValue::new(None::<(Mood, WakeStamp)>));
    let before = *seen.peek();
    if before != Some((mood, wake)) {
        seen.set(Some((mood, wake)));
        let changed = before.is_some_and(|(was, _)| was != mood);
        queue_effect(move || woken(plays, spec, mood, changed));
    }
    Life {
        blink: plays.blink.key(),
        breathe: plays.breathe.key(),
        wince: plays.wince.key(),
        hop: plays.hop.key(),
        drift: plays.drift.key(),
    }
}

/// A wake: stop the old blink timer, play the mood's motion if it changed, and open a new
/// awake window.
fn woken(plays: Plays, spec: PersonaSpec, mood: Mood, changed: bool) {
    if let Ok(Some(running)) = try_get(plays.blinker) {
        running.cancel();
    }
    let Ok(env) = try_get(plays.env) else {
        return;
    };
    let level = env.resolved.motion;
    if level == MotionLevel::Reduced {
        return;
    }
    if changed {
        match mood {
            Mood::Wince => plays.wince.play(),
            Mood::Happy => plays.hop.play(),
            Mood::Asleep => plays.drift.play(),
            Mood::Idle | Mood::Attentive => {}
        }
    }
    plays.breathe.play();
    if mood.blinks() == Blinking::Still {
        return;
    }
    let window = settle(Anim::PersonaBreathe, level, StaggerIndex::default());
    let blink = settle(Anim::PersonaBlink, level, StaggerIndex::default());
    let play = plays.blink;
    let task = spawn_in(plays.scope, async move {
        let mut at = std::time::Duration::ZERO;
        for gap in spec.blink_gaps() {
            at += gap;
            if at + blink > window {
                break;
            }
            sleep(gap).await;
            play.play();
        }
    });
    let mut blinker = plays.blinker;
    if let Ok(mut slot) = blinker.try_write() {
        *slot = Some(task);
    }
}
