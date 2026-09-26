//! The accept beat (`Anim::PictureAccept`): when a picture's mood turns Happy (the password was
//! right), the whole picture, whatever its kind, lifts once and lands. An emoji also swaps in its
//! unlock face; a letter or a photo has only this. Fired after the render that saw the change and
//! put back at rest at `settle(PictureAccept)`, so nothing stays on the element (the idle-frame
//! rule). Under Reduced motion nothing is fired.

use super::mood::Mood;
use crate::appearance::MotionLevel;
use crate::components::vocab::PulseKey;
use crate::motion::{Anim, TimerPhase, use_motion_timer, use_pulse};
use crate::root::env::use_env_signal;
use crate::task::try_get;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The accept pulse for a picture in `mood`: at rest on mount (a picture mounted already happy
/// has nothing to answer), fired each time the mood changes to [`Mood::Happy`].
pub(crate) fn use_accept(mood: Mood) -> PulseKey {
    let pulse = use_pulse(Anim::PictureAccept);
    let timer = use_motion_timer(Anim::PictureAccept);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_env_signal();
    // As `use_bump_on`: the last mood lives in a plain value, not a signal written in render.
    let mut seen = use_hook(|| CopyValue::new(mood));
    if *seen.peek() != mood {
        seen.set(mood);
        if mood == Mood::Happy {
            queue_effect(move || {
                let level = try_get(env).map_or(MotionLevel::Reduced, |env| env.resolved.motion);
                if level != MotionLevel::Reduced {
                    pulse.fire();
                    timer.start(settled);
                }
            });
        }
    }
    worn(pulse.key(), timer.phase())
}

/// What the beat wears while its timer is in `phase`: the fired key while it runs, rest otherwise.
fn worn(key: PulseKey, phase: TimerPhase) -> PulseKey {
    match phase {
        TimerPhase::Running => key,
        TimerPhase::Idle | TimerPhase::Settled => PulseKey::rest(key.anim()),
    }
}
