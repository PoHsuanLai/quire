//! A share driven frame by frame from Rust, for what the stylesheet cannot reach inside an SVG
//! (design/26-DETAILS.md section 4.1): a follower of a target, and the frame a sweep or a count
//! reads, on the easing tokens' own curves (`CubicBezier::at`).

use super::level::use_level;
use crate::core::vocab::Fraction;
use crate::motion::timeline::ease::Ease;
use crate::motion::timeline::glide::{Glide, Pose};
use crate::motion::timeline::playback::Playback;
use crate::motion::timeline::use_timeline::use_timeline;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
use dioxus::prelude::*;
use std::time::Duration;

/// How a tween moves: a duration token, along `--e-out` (a tween never overshoots: an
/// overshoot needs the person's contact, R5, and nothing a tween draws is touched).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TweenSpec {
    /// How long, as the level says.
    pub duration: DurationToken,
}

/// A tween's frame: the share to draw now and how far through its move it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tween {
    pose: Pose,
    run: u32,
    ease: Ease,
}

impl Tween {
    /// The frame `playback` draws now, subscribing the caller's render to it.
    pub(crate) fn of(playback: Playback<Glide>) -> Tween {
        Tween {
            pose: playback.frame(),
            run: playback.serial(),
            ease: playback.timeline().unwrap_or_else(|| Glide::still(0)).ease,
        }
    }

    /// The share now, in thousandths (a spring may read past its target on the way; never
    /// below 0).
    pub fn now(self) -> Fraction {
        Fraction(u16::try_from(self.pose.value.max(0)).unwrap_or(u16::MAX))
    }

    /// How far along its curve the current move is, in thousandths: what a count in step with it
    /// reads, so it lands with it.
    pub fn progress(self) -> Fraction {
        self.pose.eased
    }

    /// Time since the current move started.
    pub fn elapsed(self) -> Duration {
        self.pose.elapsed
    }

    /// Whether the move has landed.
    pub fn landed(self) -> bool {
        self.pose.through.0 >= 1000
    }

    /// Which move this frame belongs to: every start, retarget or jump is a new run.
    pub(crate) fn run(self) -> u32 {
        self.run
    }
}

/// The share of a tween that follows `target`: it stands there on mount, and each time `target`
/// changes it moves there from wherever it is now, over `spec`. Under Reduced it jumps (R7). Asks
/// for frames only while it moves (R3).
pub fn use_tween(target: Fraction, spec: TweenSpec) -> Fraction {
    let env = use_level();
    // The share drawn last, where a retarget starts from (R10).
    let mut drawn = use_hook(|| CopyValue::new(i64::from(target.0)));
    let mut plan = use_hook(|| CopyValue::new((target, Glide::still(i64::from(target.0)))));
    if plan.peek().0 != target {
        let level = env.now();
        let to = i64::from(target.0);
        let glide = match level {
            MotionLevel::Reduced => Glide::still(to),
            MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => Glide::between(
                *drawn.peek(),
                to,
                spec.duration.duration(level),
                EasingToken::Out.easing(level),
            ),
        };
        plan.set((target, glide));
    }
    let pose = use_timeline(plan.peek().1);
    drawn.set(pose.value);
    Fraction(u16::try_from(pose.value.max(0)).unwrap_or(u16::MAX))
}
