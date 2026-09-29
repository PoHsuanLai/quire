//! When an animation has finished, computed rather than observed: Blitz dispatches no
//! `animationend` (design/05-MOTION.md section 7.1).

use super::anim::Anim;
use crate::core::time::FRAME_SLACK;
use crate::style::appearance::motion::MotionLevel;
use std::time::Duration;

/// `duration(anim, level) + FRAME_SLACK`. Worked values, Standard: `settle(RowOut)` 184 ms,
/// `settle(Heal)` 284 ms; Reduced: every settle is 184 ms.
pub fn settle(anim: Anim, level: MotionLevel) -> Duration {
    anim.recipe().duration.duration(level) + FRAME_SLACK
}
