//! The frame timer under the orb's turn: it runs only while the orb is active and motion is not
//! reduced, and costs nothing otherwise (the machine has no wake while the glows stand).

use super::model::Turn;
use super::step::{Glow, GlowIn};
use dioxus::prelude::*;
use ds_core::vocab::Activity;
use ds_motion::detail::level::use_level;
use ds_motion::machine::use_machine;
use ds_style::appearance::motion::MotionLevel;
use std::time::Duration;

/// Where the glows are now. While `activity` is `Active` (and motion is not `Reduced`) they turn
/// once per `period`, a frame every `FRAME_TICK`; the moment it is `Inactive` they hold where they
/// stand, so an idle orb wakes nothing. Under `Reduced` they stand at the start.
pub(crate) fn use_turn(activity: Activity, period: Duration) -> Turn {
    let level = use_level();
    let glow = use_machine(|_| Glow::default(), (), || (), |(), _| {});
    let moving = matches!(activity, Activity::Active) && level.now() != MotionLevel::Reduced;
    glow.send_from_render(GlowIn::Spin(moving.then_some(period)));
    match level.now() {
        MotionLevel::Reduced => Turn::default(),
        _ => glow.state().read().turn,
    }
}
