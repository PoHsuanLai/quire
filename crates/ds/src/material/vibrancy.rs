//! The vibrancy boost baked into a flat tint (the macOS polish pass, 2026-09-24).
//!
//! macOS vibrancy saturates and brightens what shows through a material. Blitz can neither
//! saturate nor filter what is behind a surface (spike S15, S16: no `backdrop-filter`, no
//! `saturate()`), so the boost is precomputed into the tint colour instead: in OKLab, chroma is
//! raised by [`CHROMA_GAIN`] and, in the light scheme, lightness by [`LIGHT_LIFT`]. The dark
//! scheme keeps its lightness so the light ink keeps its contrast. The boost is scaled at render
//! time by the `appearance.material_vibrancy` key (percent, default 100) through the
//! `--m-vibrancy` input the root writes: 0 is the flat section 17.2 tint.

use crate::appearance::Scheme;
use crate::tokens::Hex;

/// Chroma multiplier at full vibrancy: 1.4.
pub const CHROMA_GAIN: f64 = 1.4;
/// OKLab lightness added at full vibrancy in the light scheme: .012.
pub const LIGHT_LIFT: f64 = 0.012;

/// `hex` with the vibrancy boost for `scheme` baked in.
pub fn boosted(hex: Hex, scheme: Scheme) -> Hex {
    let [l, a, b] = oklab(hex);
    let lift = match scheme {
        Scheme::Light => LIGHT_LIFT,
        Scheme::Dark => 0.0,
    };
    srgb([(l + lift).min(1.0), a * CHROMA_GAIN, b * CHROMA_GAIN])
}

/// sRGB bytes to OKLab (Ottosson's published matrices).
fn oklab(Hex([r, g, b]): Hex) -> [f64; 3] {
    let (r, g, b) = (linear(r), linear(g), linear(b));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// OKLab back to sRGB bytes, clamped to the gamut.
fn srgb([l, a, b]: [f64; 3]) -> Hex {
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_.powi(3), m_.powi(3), s_.powi(3));
    let r = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let g = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;
    Hex([byte(r), byte(g), byte(b)])
}

fn linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn byte(linear: f64) -> u8 {
    let c = linear.clamp(0.0, 1.0);
    let encoded = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    // In range after the clamp: 0..=255.
    (encoded * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{boosted, oklab, srgb};
    use crate::appearance::Scheme;
    use crate::tokens::Hex;

    #[test]
    fn oklab_round_trips_every_tint() {
        for hex in [
            Hex([248, 249, 246]),
            Hex([255, 255, 255]),
            Hex([21, 24, 20]),
            Hex([42, 47, 40]),
        ] {
            assert_eq!(srgb(oklab(hex)), hex);
        }
    }

    #[test]
    fn the_boost_lifts_the_light_tint_and_keeps_the_dark_ones_lightness() {
        let light = boosted(Hex([248, 249, 246]), Scheme::Light);
        assert!(
            light
                .0
                .iter()
                .zip([248, 249, 246])
                .all(|(got, was)| *got >= was),
            "{light:?}"
        );
        assert_ne!(light, Hex([248, 249, 246]));
        let dark = boosted(Hex([21, 24, 20]), Scheme::Dark);
        let [l0, ..] = oklab(Hex([21, 24, 20]));
        let [l1, ..] = oklab(dark);
        assert!((l0 - l1).abs() < 0.005, "{l0} {l1}");
        // The dark raise's green is the more saturated for the boost.
        assert!(dark.0[1] >= 24, "{dark:?}");
    }
}
