//! A share driven frame by frame from Rust, for what the stylesheet cannot reach inside an SVG
//! (design/26-DETAILS.md section 4.1): a follower of a target on the easing tokens' own curves
//! (`CubicBezier::at`). A determinate value that changes (a battery ring's arc, a slash drawn
//! on) moves to its new value and nothing sweeps in on first show (design/30 section 1.3).

use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// How a tween moves: a duration token along an easing token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TweenSpec {
    /// How long, as the level says.
    pub duration: DurationToken,
    /// The curve: `--e-linear` for a value a person reads as it moves.
    pub easing: EasingToken,
}

/// The share of a tween that follows `target`: it stands there on mount, and each time `target`
/// changes it moves there from wherever it is now, over `spec`. Under Reduced it jumps (R7). Asks
/// for frames only while it moves (R3).
#[cfg(feature = "dioxus")]
pub fn use_tween(target: ds_core::vocab::Fraction, spec: TweenSpec) -> ds_core::vocab::Fraction {
    use super::level::use_level;
    use crate::timeline::glide::Glide;
    use crate::timeline::use_timeline::use_timeline;
    use dioxus::prelude::*;
    use ds_style::appearance::motion::MotionLevel;

    let env = use_level();
    // The share drawn last, where a retarget starts from (R10).
    let mut drawn = use_hook(|| CopyValue::new(i64::from(target.0)));
    let mut plan = use_hook(|| CopyValue::new((target, Glide::still(i64::from(target.0)))));
    if plan.peek().0 != target {
        let level = env.now();
        let to = i64::from(target.0);
        let glide = match level {
            MotionLevel::Reduced => Glide::still(to),
            MotionLevel::Standard => Glide::between(
                *drawn.peek(),
                to,
                spec.duration.duration(level),
                spec.easing.easing(level),
            ),
        };
        plan.set((target, glide));
    }
    let pose = use_timeline(plan.peek().1);
    drawn.set(pose.value);
    ds_core::vocab::Fraction(u16::try_from(pose.value.max(0)).unwrap_or(u16::MAX))
}
