//! A colour muted: its OKLCH chroma scaled to .55, lightness and hue kept. S writes it as
//! `filter: saturate(.55)` on an account not in view; Blitz paints no `filter` (O-26), so the
//! colour is computed here, once, for the account tile and for `Avatar { muting }`.
//!
//! Keeping the hue keeps the claim "this is that account" (design/00-PRINCIPLES.md, colour is a
//! claim); taking chroma away says it is not the one in view. A grey would drop the first fact,
//! and every muted account would look alike.

use crate::style::tokens::hex::{Colour, Hex};
use ds_core::colour::{oklab::Oklab, srgb::Srgb};

/// How much of its chroma a muted colour keeps: S's `saturate(.55)`.
pub(crate) const MUTED_CHROMA: f64 = 0.55;

/// `colour` muted, its alpha kept.
pub(crate) fn muted(colour: Colour) -> Colour {
    match colour {
        Colour::Solid(hex) => Colour::Solid(desaturated(hex)),
        Colour::Alpha(hex, alpha) => Colour::Alpha(desaturated(hex), alpha),
    }
}

/// `hex` with its OKLCH chroma scaled by [`MUTED_CHROMA`], lightness and hue kept.
pub(crate) fn desaturated(hex: Hex) -> Hex {
    let Oklab { l, a, b } = Oklab::from(Srgb::from(hex));
    let muted = Oklab {
        l,
        a: a * MUTED_CHROMA,
        b: b * MUTED_CHROMA,
    };
    Hex::from(Srgb::from(muted))
}

#[cfg(test)]
mod tests {
    use super::{desaturated, muted};
    use crate::style::tokens::hex::{Alpha, Colour, Hex};
    use ds_core::colour::{oklab::Oklab, srgb::Srgb};

    fn oklab(hex: Hex) -> [f64; 3] {
        let Oklab { l, a, b } = Oklab::from(Srgb::from(hex));
        [l, a, b]
    }

    #[test]
    fn a_grey_stays_grey_and_a_hue_loses_chroma() {
        let grey = Hex([0x80, 0x80, 0x80]);
        assert_eq!(desaturated(grey), grey);
        let violet = Hex([0x5b, 0x4f, 0xc4]);
        let [l0, a0, b0] = oklab(violet);
        let [l1, a1, b1] = oklab(desaturated(violet));
        let chroma = |a: f64, b: f64| a.hypot(b);
        assert!((l1 - l0).abs() < 0.01, "lightness kept: {l0} {l1}");
        let kept = chroma(a1, b1) / chroma(a0, b0);
        assert!((kept - 0.55).abs() < 0.03, "chroma kept {kept}");
    }

    #[test]
    fn muting_keeps_the_alpha() {
        let violet = Hex([0x5b, 0x4f, 0xc4]);
        assert_eq!(
            muted(Colour::Alpha(violet, Alpha(500))),
            Colour::Alpha(desaturated(violet), Alpha(500))
        );
    }
}
