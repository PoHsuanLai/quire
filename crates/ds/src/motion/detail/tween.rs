//! A share driven frame by frame from Rust, for what the stylesheet cannot reach inside an SVG
//! (design/26-DETAILS.md section 4.1): the frame driver under Sweep, CountUp and the success
//! check, on the easing tokens' own curves (`CubicBezier::at`).

use super::level::use_level;
use crate::core::vocab::Fraction;
use crate::motion::timeline::ease::Ease;
use crate::motion::timeline::glide::{Glide, Pose};
use crate::motion::timeline::playback::{Playback, use_playback};
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
use dioxus::core::queue_effect;
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

/// A tween that follows `target`: it stands there on mount, and each time `target` changes it
/// moves there from wherever it is now, over `spec`. Under Reduced it jumps (R7). Asks for frames
/// only while it moves (R3).
pub fn use_tween(target: Fraction, spec: TweenSpec) -> Tween {
    let playback = use_playback(Glide::still(i64::from(target.0)));
    let env = use_level();
    let mut seen = use_hook(|| CopyValue::new(target));
    if *seen.peek() != target {
        seen.set(target);
        queue_effect(move || {
            let level = env.now();
            let to = i64::from(target.0);
            match level {
                MotionLevel::Reduced => playback.play(Glide::still(to)),
                MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => {
                    playback.play(Glide::between(
                        playback.peek().map_or(to, |pose| pose.value),
                        to,
                        spec.duration.duration(level),
                        EasingToken::Out.easing(level),
                    ))
                }
            }
        });
    }
    Tween::of(playback)
}
