//! An entrance played once as a surface mounts: `Entering` until the animation settles, then
//! `Present` (design/05-MOTION.md section 7, timers instead of `animationend`). Every floating
//! surface quire draws enters this way, and a consumer's own surface (a control-center pane, a
//! card of its own) can too without restating the timer (sill FINDINGS Q80).

use crate::motion::anim::Anim;
use crate::motion::presence::Presence;
use crate::motion::timer::{TimerPhase, use_motion_timer};
use dioxus::prelude::*;

/// `data-presence` for a surface that plays `anim` as it mounts: `Presence::Entering` until
/// `settle(anim)`, then `Presence::Present`. A surface that leaves is removed at once, so there
/// is no leaving state here; one that must leave with an exit animation owns that itself.
pub fn use_entrance(anim: Anim) -> Presence {
    let timer = use_motion_timer(anim);
    use_hook(|| timer.start(EventHandler::new(|()| {})));
    match timer.phase() {
        TimerPhase::Settled => Presence::Present,
        TimerPhase::Idle | TimerPhase::Running => Presence::Entering,
    }
}
