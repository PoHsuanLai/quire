//! What an accent is measured against: the card's grounds, and how far a colour stands out on
//! them.

use crate::appearance::theme::Scheme;
use crate::tokens::{
    colour::ColourToken,
    hex::{Colour, Hex},
};
use ds_core::colour::contrast::ratio;

/// The card's grounds an accent may lie on: `--paper`, `--surface`, `--surface-2`, `--raise`.
pub fn card_grounds(scheme: Scheme) -> [Hex; 4] {
    [
        ColourToken::Paper,
        ColourToken::Surface,
        ColourToken::Surface2,
        ColourToken::Raise,
    ]
    .map(|token| solid(token.value(scheme)))
}

/// The card's `--ink`, the text on a wash.
pub fn card_ink(scheme: Scheme) -> Hex {
    solid(ColourToken::Ink.value(scheme))
}

/// The WCAG ratio between two colours; a pair that cannot be measured reads as 1:1, failing.
pub(crate) fn contrast(fore: Hex, back: Hex) -> f64 {
    ratio(&fore.css(), &back.css()).unwrap_or(1.0)
}

/// The least ratio `fore` reaches against any of `grounds`.
pub(crate) fn least(fore: impl Fn(Hex) -> Hex, against: &[Hex]) -> f64 {
    against
        .iter()
        .map(|&ground| contrast(fore(ground), ground))
        .fold(f64::INFINITY, f64::min)
}

/// The hex under a card token; the four grounds and the ink are all solid.
fn solid(colour: Colour) -> Hex {
    match colour {
        Colour::Solid(hex) | Colour::Alpha(hex, _) => hex,
    }
}
