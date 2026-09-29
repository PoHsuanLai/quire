//! A keyframe played once per new cue of one moment: Shake for a Failure (design/26-DETAILS.md
//! R6), and a content cross-fade for a part's in-place change. The same
//! cue never replays; a new one replays the identical motion, never a bigger one; Reduced plays
//! nothing (R7), the still state carries it (R8).

use super::cue::Cue;
use super::level::use_level;
use super::moment::Moment;
use crate::motion::pulse_key::PulseKey;
use crate::motion::{
    anim::Anim,
    pulse::use_pulse,
    timer::{TimerPhase, use_motion_timer},
};
use crate::style::appearance::motion::MotionLevel;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// `anim` once each time `cue` is a new change to one of `moments`, at rest otherwise (and with
/// no cue at all). Render the key on the HTML wrapper that moves (`PulseKey::attrs`).
fn use_once(anim: Anim, moments: &'static [Moment], cue: Option<Cue>) -> PulseKey {
    let pulse = use_pulse(anim);
    let timer = use_motion_timer(anim);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_level();
    let serial = cue.map(Cue::serial);
    let mut seen = use_hook(|| CopyValue::new(serial));
    if *seen.peek() != serial {
        seen.set(serial);
        let reduced = env.now() == MotionLevel::Reduced;
        let named = cue.is_some_and(|cue| moments.contains(&cue.moment()));
        if named && !reduced {
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
    use_once(Anim::ShakeX, &[Moment::Failure], Some(cue))
}

/// The incoming content's `fade` at `--t-quick` once per new Preview, Change or Failure cue: what
/// a part that swaps its content in place (the preview pane's media) plays instead of its
/// entrance. Nothing under Reduced: the content snaps (R7).
pub fn use_cross_fade(cue: Option<Cue>) -> PulseKey {
    use_once(
        Anim::MorphFadeIn,
        &[Moment::Preview, Moment::Change, Moment::Failure],
        cue,
    )
}
