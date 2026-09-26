//! A share driven frame by frame from Rust, for what the stylesheet cannot reach inside an SVG
//! (design/26-DETAILS.md section 4.1): the frame driver under Sweep, CountUp and the success
//! check, on the easing tokens' own curves (`CubicBezier::at`).

use super::glide::{Glide, Pose};
use super::level::use_level;
use super::motor::use_motor;
use super::touch::Contact;
use crate::appearance::MotionLevel;
use crate::components::Fraction;
use crate::tokens::{DurationToken, EasingToken};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use std::time::Duration;

/// The curve a tween follows. An overshoot needs the person's [`Contact`] (R5): a remote change
/// cannot ask for one.
///
/// ```
/// use dioxus::prelude::{Event, MouseData};
/// use ds::DurationToken;
/// use ds::detail::{Contact, Ease, TweenSpec};
///
/// fn flicked(event: &Event<MouseData>) -> TweenSpec {
///     TweenSpec { duration: DurationToken::Quick, ease: Ease::Spring(Contact::from_event(event)) }
/// }
/// ```
///
/// A spring with no contact does not compile:
///
/// ```compile_fail,E0308
/// use ds::DurationToken;
/// use ds::detail::{Ease, TweenSpec};
///
/// let remote = TweenSpec { duration: DurationToken::Quick, ease: Ease::Spring };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ease {
    /// `--e-out`.
    Out,
    /// `--e-in-out`.
    InOut,
    /// `--e-linear`.
    Linear,
    /// `--e-exit`.
    Exit,
    /// `--e-spring`, on the element the person touched.
    Spring(Contact),
}

impl Ease {
    /// The token this curve is.
    pub fn token(self) -> EasingToken {
        match self {
            Ease::Out => EasingToken::Out,
            Ease::InOut => EasingToken::InOut,
            Ease::Linear => EasingToken::Linear,
            Ease::Exit => EasingToken::Exit,
            Ease::Spring(_) => EasingToken::Spring,
        }
    }
}

/// How a tween moves: a duration token and a curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TweenSpec {
    /// How long, as the level says.
    pub duration: DurationToken,
    /// Along which curve.
    pub ease: Ease,
}

/// A tween's frame: the share to draw now and how far through its move it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tween {
    pose: Pose,
}

impl Tween {
    pub(crate) fn from_pose(pose: Pose) -> Tween {
        Tween { pose }
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
        self.pose.run
    }
}

/// A tween that follows `target`: it stands there on mount, and each time `target` changes it
/// moves there from wherever it is now, over `spec`. Under Reduced it jumps (R7). Asks for frames
/// only while it moves (R3).
pub fn use_tween(target: Fraction, spec: TweenSpec) -> Tween {
    let motor = use_motor(i64::from(target.0));
    let env = use_level();
    let mut seen = use_hook(|| CopyValue::new(target));
    if *seen.peek() != target {
        seen.set(target);
        queue_effect(move || {
            let level = env.now();
            let to = i64::from(target.0);
            match level {
                MotionLevel::Reduced => motor.snap(to),
                MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => {
                    motor.play(Glide {
                        from: motor.peek().value,
                        to,
                        length: spec.duration.duration(level),
                        easing: spec.ease.token().easing(level),
                    })
                }
            }
        });
    }
    Tween::from_pose(motor.pose())
}
