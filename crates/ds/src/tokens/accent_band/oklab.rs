//! How far apart two swatches look: Euclidean distance in OKLab, for the picker's distinctness
//! gate.

use crate::tokens::hex::Hex;

/// The OKLab distance between two colours (0 is identical; about 0.02 is a just-noticeable step).
pub fn distance(one: Hex, other: Hex) -> f64 {
    let [l1, a1, b1] = oklab(one);
    let [l2, a2, b2] = oklab(other);
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

fn linear(byte: u8) -> f64 {
    let channel = f64::from(byte) / 255.0;
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab(colour: Hex) -> [f64; 3] {
    let [r, g, b] = colour.0.map(linear);
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}
