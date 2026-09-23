//! When an animation has finished, computed rather than observed: Blitz dispatches no
//! `animationend` (design/05-MOTION.md section 7.1).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::anim::Anim;
use crate::appearance::MotionLevel;
use crate::components::vocab::StaggerIndex;
use std::time::Duration;

/// `duration(anim, level) + index x stagger(level) + FRAME_SLACK`.
///
/// Heal steps by `HealStep` (18 ms) rather than `--stagger`. Worked values, Post Standard:
/// `settle(Fold)` 454 ms, `settle(FoldHeavy)` 517 ms, `settle(Curl)` 594 ms; Reduced: every
/// settle is 94 ms.
pub fn settle(anim: Anim, level: MotionLevel, index: StaggerIndex) -> Duration {
    todo!()
}
