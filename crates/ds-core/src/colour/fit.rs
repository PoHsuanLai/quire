//! `oklch(l c h)` into 8-bit sRGB the way the approved CSS prototype does it: chroma stepped
//! down by [`GAMUT_STEP`] until the colour fits (within half a thousandth), each channel
//! rounded with JavaScript's `Math.round`. The Space palette, the accent band and the emoji
//! discs all draw their derived colours through it, so they agree with the prototype to the
//! byte.

use super::oklab::{Oklab, Oklch};
use super::srgb::{LinearRgb, encode};

/// How far the fit drops chroma while the colour is outside sRGB.
const GAMUT_STEP: f64 = 0.002;

/// `Math.round`: halves go up, including for negatives. Rust's `round` sends halves away from
/// zero, which disagrees at -1.5.
pub fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

/// `oklch(lightness chroma hue)`, hue in degrees, as 8-bit sRGB channels, its chroma reduced
/// until it fits sRGB.
pub fn oklch_bytes(lightness: f64, chroma: f64, hue: f64) -> [u8; 3] {
    let LinearRgb(channels) = linear(lightness, fit(lightness, chroma, hue), hue);
    channels.map(|channel| channel_byte(encode(channel)))
}

/// The same colour as `#rrggbb`.
pub fn oklch_hex(lightness: f64, chroma: f64, hue: f64) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(7);
    out.push('#');
    for byte in oklch_bytes(lightness, chroma, hue) {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn linear(lightness: f64, chroma: f64, hue: f64) -> LinearRgb {
    LinearRgb::from(Oklab::from(Oklch::from_degrees(lightness, chroma, hue)))
}

fn in_gamut(LinearRgb(channels): LinearRgb) -> bool {
    channels
        .iter()
        .all(|channel| (-0.0005..=1.0005).contains(channel))
}

fn fit(lightness: f64, chroma: f64, hue: f64) -> f64 {
    let mut chroma = chroma;
    while chroma > 0.0 && !in_gamut(linear(lightness, chroma, hue)) {
        chroma -= GAMUT_STEP;
    }
    chroma.max(0.0)
}

fn channel_byte(encoded: f64) -> u8 {
    let rounded = js_round(encoded * 255.0);
    if (0.0..=255.0).contains(&rounded) {
        // `js_round` produced a whole number in range.
        rounded as u8
    } else if rounded < 0.0 {
        0
    } else {
        255
    }
}
