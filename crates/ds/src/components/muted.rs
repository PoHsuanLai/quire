//! A colour muted: its OKLCH chroma scaled to .55, lightness and hue kept. S writes it as
//! `filter: saturate(.55)` on an account not in view; Blitz paints no `filter` (O-26), so the
//! colour is computed here, once, for the account tile and for `Avatar { muting }`.
//!
//! Keeping the hue keeps the claim "this is that account" (design/00-PRINCIPLES.md, colour is a
//! claim); taking chroma away says it is not the one in view. A grey would drop the first fact,
//! and every muted account would look alike.

use crate::tokens::{Colour, Hex};

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
    let [l, a, b] = oklab(hex);
    from_oklab([l, a * MUTED_CHROMA, b * MUTED_CHROMA])
}

/// sRGB to OKLab (Björn Ottosson's matrices, the inverse of `space::palette`'s).
pub(crate) fn oklab(Hex(rgb): Hex) -> [f64; 3] {
    let [r, g, b] = rgb.map(|channel| linear(f64::from(channel) / 255.0));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// OKLab back to 8-bit sRGB, clamped into gamut.
fn from_oklab([lightness, a, b]: [f64; 3]) -> Hex {
    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    let rgb = [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ];
    // Rounded and clamped to 0..=255 first, so the cast cannot truncate.
    Hex(rgb.map(|channel| (encode(channel) * 255.0).round().clamp(0.0, 255.0) as u8))
}

/// sRGB transfer, decoding.
fn linear(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB transfer, encoding.
fn encode(channel: f64) -> f64 {
    let channel = channel.clamp(0.0, 1.0);
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::{desaturated, muted, oklab};
    use crate::tokens::{Alpha, Colour, Hex};

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
