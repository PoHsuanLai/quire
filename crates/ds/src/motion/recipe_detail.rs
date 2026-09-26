//! The recipes of the small-state details' animations (design/26-DETAILS.md section 3.2): the
//! glyph morph's and the rolling digits' layers, the remote seal and the one-shot nudge. Every
//! one plays once; the grammar has no loop outside a bounded pending step.

use super::recipe::{Fill, Iteration, Recipe, recipe};
use crate::tokens::{DurationToken, EasingToken};

/// `morph-in` at `--t-quick --e-out`: the incoming glyph of a DownUp or OffUp morph.
pub(super) const MORPH_IN: Recipe = recipe(
    "morph-in",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Backwards,
    Iteration::Once,
);

/// `morph-out` at `--t-quick --e-out`: the outgoing glyph of a DownUp morph, held gone until
/// its layer is removed.
pub(super) const MORPH_OUT: Recipe = recipe(
    "morph-out",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Forwards,
    Iteration::Once,
);

/// `fade` at `--t-quick --e-out`: the incoming glyph of a cross-fade.
pub(super) const MORPH_FADE_IN: Recipe = recipe(
    "fade",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Backwards,
    Iteration::Once,
);

/// `morph-fade-out` at `--t-quick --e-out`: the outgoing glyph of a cross-fade.
pub(super) const MORPH_FADE_OUT: Recipe = recipe(
    "morph-fade-out",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Forwards,
    Iteration::Once,
);

/// `roll-in` at `--t-quick --e-out`: a changed digit rolling into place.
pub(super) const ROLL_IN: Recipe = recipe(
    "roll-in",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Backwards,
    Iteration::Once,
);

/// `roll-out` at `--t-quick --e-out`: the digit it replaces rolling away.
pub(super) const ROLL_OUT: Recipe = recipe(
    "roll-out",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::Forwards,
    Iteration::Once,
);

/// `gulp` at `--t-big --e-out`: a success seal that nobody touched (R5: the spring is spent only
/// on contact, where `Anim::Gulp` plays).
pub(super) const SEAL_OUT: Recipe = recipe(
    "gulp",
    DurationToken::Big,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `nudge-up` at `--t-nudge --e-out`: attention, once per request (R6).
pub(super) const NUDGE_UP: Recipe = recipe(
    "nudge-up",
    DurationToken::Nudge,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);
