//! How long a Rust-driven symbol motion takes, on the same duration and easing tokens as the
//! stylesheet's (design/35-SYMBOL-EFFECTS.md section 3): the part driver's counterpart of an
//! `Anim`'s recipe row, since a motion inside an SVG has no keyframes for the stylesheet to play.

use super::route::PartRun;
use crate::detail::tween::TweenSpec;
use crate::timeline::cycle::{Cycle, Repeat};
use ds_style::appearance::motion::MotionLevel;
use ds_style::icon::parts::PartGesture;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// How the strokes draw on or off.
pub const DRAW: TweenSpec = TweenSpec {
    duration: DurationToken::Move,
    easing: EasingToken::Out,
};

/// The curve of one cycle: layers step at a constant rate, a turn and a swing ease in and out,
/// the rest decelerate into place.
fn easing(gesture: PartGesture, run: PartRun) -> EasingToken {
    match (gesture, run) {
        (PartGesture::Layers, _) => EasingToken::Linear,
        (PartGesture::Turn, PartRun::While(_)) => EasingToken::Linear,
        (PartGesture::Turn, PartRun::Once(_)) | (PartGesture::Ring, PartRun::While(_)) => {
            EasingToken::InOut
        }
        _ => EasingToken::Out,
    }
}

/// The cycle that plays `gesture` for `run` at `level`: once at `--t-big`, or forever at
/// `--t-turn`.
pub fn cycle(gesture: PartGesture, run: PartRun, level: MotionLevel) -> Cycle {
    let (token, repeat) = match run {
        PartRun::Once(_) => (DurationToken::Big, Repeat::Once),
        PartRun::While(_) => (DurationToken::Turn, Repeat::Forever),
    };
    Cycle {
        period: token.duration(level),
        easing: easing(gesture, run).easing(level),
        repeat,
    }
}
