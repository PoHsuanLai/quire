//! A status glyph's slash, drawn on and off over `--t-quick --e-out` (design/26 5.1: Unavailable
//! is `MorphGlyph{Slash}`), standing still on the first frame and jumping under Reduced (R7).

use ds_core::vocab::Fraction;
use ds_motion::detail::{
    morph::Slashed,
    tween::{TweenSpec, use_tween},
};
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// How the slash moves.
const DRAW: TweenSpec = TweenSpec {
    duration: DurationToken::Quick,
    easing: EasingToken::Out,
};

/// How much of the slash is drawn this frame, in thousandths.
pub(crate) fn use_slash(slashed: Slashed) -> Fraction {
    let target = match slashed {
        Slashed::On => Fraction(1000),
        Slashed::Off => Fraction(0),
    };
    use_tween(target, DRAW)
}
