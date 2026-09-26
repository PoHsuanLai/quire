//! A keyframe played once per new cue of one moment: Shake for a Failure, Nudge for Attention
//! (design/26-DETAILS.md R6). The same cue never replays; a new one replays the identical motion,
//! never a bigger one; Reduced plays nothing (R7), the still state carries it (R8).

use super::cue::Cue;
use super::level::use_level;
use super::moment::Moment;
use crate::appearance::MotionLevel;
use crate::components::vocab::PulseKey;
use crate::motion::{Anim, TimerPhase, use_motion_timer, use_pulse};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// `anim` once each time `cue` is a new `moment`, at rest otherwise. Render the key on the HTML
/// wrapper that moves (`PulseKey::attrs`).
fn use_once(anim: Anim, moment: Moment, cue: Cue) -> PulseKey {
    let pulse = use_pulse(anim);
    let timer = use_motion_timer(anim);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_level();
    let mut seen = use_hook(|| CopyValue::new(cue.serial()));
    if *seen.peek() != cue.serial() {
        seen.set(cue.serial());
        let reduced = env.now() == MotionLevel::Reduced;
        if cue.moment() == moment && !reduced {
            queue_effect(move || {
                pulse.fire();
                timer.start(settled);
            });
        }
    }
    match timer.phase() {
        TimerPhase::Running => pulse.key(),
        TimerPhase::Idle | TimerPhase::Settled => PulseKey::rest(anim),
    }
}

/// `shake-x` once per new Failure cue (R6); the same amplitude every time; nothing under Reduced.
pub fn use_shake(cue: Cue) -> PulseKey {
    use_once(Anim::ShakeX, Moment::Failure, cue)
}

/// `nudge-up` once per new Attention cue (R6); nothing under Reduced.
pub fn use_nudge(cue: Cue) -> PulseKey {
    use_once(Anim::NudgeUp, Moment::Attention, cue)
}
