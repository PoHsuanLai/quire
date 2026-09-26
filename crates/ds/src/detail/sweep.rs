//! Sweep: an arc or a bar from one share to another (design/26-DETAILS.md section 3.2).

use super::cue::Cue;
use super::glide::Glide;
use super::level::use_level;
use super::moment::Moment;
use super::motor::{Motor, use_motor};
use super::tween::Tween;
use crate::appearance::MotionLevel;
use crate::components::Fraction;
use crate::tokens::{DurationToken, EasingToken};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// An arc's or a bar's share this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sweep {
    tween: Tween,
}

impl Sweep {
    /// The share to draw now: an SVG arc's span or a bar's `--f`.
    pub fn share(self) -> Fraction {
        self.tween.now()
    }

    /// The frame as a tween, for a count in step with it.
    pub fn tween(self) -> Tween {
        self.tween
    }
}

/// How a sweep answers a moment: from where, over which token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SweepPlan {
    /// From zero over `--t-sweep` (Appear).
    FromZero,
    /// From the current share over `--t-quick` (Change, Progress).
    FromHere,
    /// Stand at the level with no frames.
    Stand,
}

/// Which plan a moment at `level` asks for: Reduced jumps (R7).
pub(crate) fn plan(moment: Moment, level: MotionLevel) -> SweepPlan {
    match (moment, level) {
        (_, MotionLevel::Reduced) => SweepPlan::Stand,
        (Moment::Appear, _) => SweepPlan::FromZero,
        (Moment::Change | Moment::Progress, _) => SweepPlan::FromHere,
        (
            Moment::Rest
            | Moment::Pending
            | Moment::Success
            | Moment::Failure
            | Moment::Select
            | Moment::Attention
            | Moment::Unavailable
            | Moment::Preview
            | Moment::Dismiss,
            _,
        ) => SweepPlan::Stand,
    }
}

/// Sweep to `level` as `cue` says: Appear sweeps from zero over `--t-sweep`, Change and Progress
/// from the current share over `--t-quick`, anything else stands at the level with no frames.
/// A level that changes with no new cue stands there too. Retargets from where it is (R10).
pub fn use_sweep(level: Fraction, cue: Cue) -> Sweep {
    let env = use_level();
    let motion = env.now();
    let start = match plan(cue.moment(), motion) {
        SweepPlan::FromZero => 0,
        SweepPlan::FromHere | SweepPlan::Stand => i64::from(level.0),
    };
    let motor = use_motor(start);
    let mut seen = use_hook(|| CopyValue::new(None::<(u32, Fraction)>));
    let now = Some((cue.serial(), level));
    if *seen.peek() != now {
        let fresh = seen.peek().is_none_or(|(serial, _)| serial != cue.serial());
        seen.set(now);
        queue_effect(move || {
            let motion = env.now();
            let moment = if fresh { cue.moment() } else { Moment::Rest };
            sweep(motor, plan(moment, motion), level, motion);
        });
    }
    Sweep {
        tween: Tween::from_pose(motor.pose()),
    }
}

/// Start `plan` towards `level` at motion level `motion`.
fn sweep(motor: Motor, plan: SweepPlan, level: Fraction, motion: MotionLevel) {
    let to = i64::from(level.0);
    let (from, token) = match plan {
        SweepPlan::Stand => return motor.snap(to),
        SweepPlan::FromZero => (0, DurationToken::Sweep),
        SweepPlan::FromHere => (motor.peek().value, DurationToken::Quick),
    };
    motor.play(Glide {
        from,
        to,
        length: token.duration(motion),
        easing: EasingToken::Out.easing(motion),
    });
}

#[cfg(test)]
mod tests {
    use super::{SweepPlan, plan};
    use crate::appearance::MotionLevel;
    use crate::detail::Moment;

    #[test]
    fn each_moment_sweeps_its_own_way_and_reduced_stands() {
        const CASES: &[(Moment, MotionLevel, SweepPlan)] = &[
            (Moment::Appear, MotionLevel::Standard, SweepPlan::FromZero),
            (Moment::Appear, MotionLevel::Calm, SweepPlan::FromZero),
            (Moment::Change, MotionLevel::Standard, SweepPlan::FromHere),
            (Moment::Progress, MotionLevel::Extra, SweepPlan::FromHere),
            (Moment::Rest, MotionLevel::Standard, SweepPlan::Stand),
            (Moment::Success, MotionLevel::Standard, SweepPlan::Stand),
            (Moment::Appear, MotionLevel::Reduced, SweepPlan::Stand),
            (Moment::Change, MotionLevel::Reduced, SweepPlan::Stand),
        ];
        for &(moment, level, want) in CASES {
            assert_eq!(plan(moment, level), want, "{moment:?} {level:?}");
        }
    }
}
