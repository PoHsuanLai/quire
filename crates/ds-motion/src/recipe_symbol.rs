//! The symbol effects' recipes (design/35-SYMBOL-EFFECTS.md section 2): the whole-icon motions the
//! stylesheet plays on a symbol's wrapper. A loop is the same keyframes as its once, forever, at
//! `--t-turn` where an effect repeating every `--t-big` would be restless.

use super::recipe::{Fill, Iteration, Recipe, recipe};
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// `symbol-bounce`: a hop and a small rebound, once.
pub(super) const BOUNCE: Recipe = recipe(
    "symbol-bounce",
    DurationToken::Big,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `symbol-bounce`, forever.
pub(super) const BOUNCE_LOOP: Recipe = recipe(
    "symbol-bounce",
    DurationToken::Big,
    EasingToken::Out,
    Fill::None,
    Iteration::Infinite,
);

/// `symbol-pulse`: a dip in opacity and back, once.
pub(super) const PULSE: Recipe = recipe(
    "symbol-pulse",
    DurationToken::Big,
    EasingToken::InOut,
    Fill::None,
    Iteration::Once,
);

/// `symbol-pulse`, forever.
pub(super) const PULSE_LOOP: Recipe = recipe(
    "symbol-pulse",
    DurationToken::Turn,
    EasingToken::InOut,
    Fill::None,
    Iteration::Infinite,
);

/// `symbol-wiggle`: a few degrees each way, once.
pub(super) const WIGGLE: Recipe = recipe(
    "symbol-wiggle",
    DurationToken::Big,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `symbol-wiggle`, forever.
pub(super) const WIGGLE_LOOP: Recipe = recipe(
    "symbol-wiggle",
    DurationToken::Turn,
    EasingToken::InOut,
    Fill::None,
    Iteration::Infinite,
);

/// `symbol-breathe`: a swell and back, once.
pub(super) const BREATHE: Recipe = recipe(
    "symbol-breathe",
    DurationToken::Big,
    EasingToken::InOut,
    Fill::None,
    Iteration::Once,
);

/// `symbol-breathe`, forever.
pub(super) const BREATHE_LOOP: Recipe = recipe(
    "symbol-breathe",
    DurationToken::Turn,
    EasingToken::InOut,
    Fill::None,
    Iteration::Infinite,
);

/// `turn`: one revolution, eased, where `Anim::Turn` is the constant-speed loop.
pub(super) const ROTATE_ONCE: Recipe = recipe(
    "turn",
    DurationToken::Big,
    EasingToken::InOut,
    Fill::None,
    Iteration::Once,
);

/// `symbol-scale-up`: grows and holds while the effect is active.
pub(super) const SCALE_UP: Recipe = recipe(
    "symbol-scale-up",
    DurationToken::Move,
    EasingToken::Out,
    Fill::Forwards,
    Iteration::Once,
);

/// `symbol-scale-down`: shrinks and holds while the effect is active.
pub(super) const SCALE_DOWN: Recipe = recipe(
    "symbol-scale-down",
    DurationToken::Move,
    EasingToken::Out,
    Fill::Forwards,
    Iteration::Once,
);

/// `morph-in` at `--t-move --e-out`: a symbol arriving.
pub(super) const APPEAR: Recipe = recipe(
    "morph-in",
    DurationToken::Move,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `morph-out` at `--t-quick --e-exit`: a symbol leaving, holding its last frame.
pub(super) const DISAPPEAR: Recipe = recipe(
    "morph-out",
    DurationToken::Quick,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);
