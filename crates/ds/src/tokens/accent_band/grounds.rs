//! What an accent is measured against: the card's grounds, and compositing a translucent role
//! over one.

use crate::appearance::theme::Scheme;
use crate::space::contrast::ratio;
use crate::tokens::{
    colour::ColourToken,
    hex::{Alpha, Colour, Hex},
};

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

/// `fore` at `alpha` over the opaque `back`, blended per 8-bit channel as a browser does.
pub fn over(fore: Hex, alpha: Alpha, back: Hex) -> Hex {
    let weight = f64::from(alpha.0.min(1000)) / 1000.0;
    let mut out = [0u8; 3];
    for ((slot, front), behind) in out.iter_mut().zip(fore.0).zip(back.0) {
        let mixed = f64::from(front) * weight + f64::from(behind) * (1.0 - weight);
        // A blend of two bytes stays within 0..=255.
        *slot = mixed.round().clamp(0.0, 255.0) as u8;
    }
    Hex(out)
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
