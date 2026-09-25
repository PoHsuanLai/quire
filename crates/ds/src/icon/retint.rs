//! Re-colouring an app icon's raster for the icon style (design/08-ICONS.md sections 2.10 and
//! 2.11; the settings `icons.style` and `icons.monochrome_tint`, design/22-SETTINGS.md 3.3).
//!
//! Pure and renderer-free: it works on an RGBA8 byte slice (straight alpha, sRGB), so the caller
//! decodes and encodes. In OKLCh, every pixel keeps its lightness and alpha:
//!
//! - [`IconStyle::Colour`] leaves the pixels alone.
//! - [`IconStyle::Muted`] scales chroma by [`MUTED_SCALE`] (0.04 / 0.07: the shipped set's cap
//!   down to the muted set's).
//! - [`IconStyle::Monochrome`] replaces hue with the tint's and chroma with the tint's, shaped
//!   by lightness (full at mid lightness, none at black or white) and by alpha (a translucent
//!   shadow takes little colour), then pulled back into sRGB.
//!
//! The quire app icons ship a neutral grey Monochrome set for exactly this call; a third-party
//! icon is re-coloured the same way, so a Monochrome dock is one hue.

use crate::Scheme;
use crate::space::{Dot, derive};

/// The style an app icon is drawn in: `icons.style`, which `sill` maps onto this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconStyle {
    /// Each app as designed; third-party icons as they come.
    #[default]
    Colour,
    /// Chroma held down (the shipped cap 0.07 scaled to 0.04).
    Muted,
    /// One hue, the tint's, tone on tone.
    Monochrome,
}

/// The hue and chroma a Monochrome icon is tinted with (OKLCh; hue in degrees).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    pub hue: f32,
    pub chroma: f32,
}

/// Muted's chroma scale: the muted set's cap over the shipped set's.
pub const MUTED_SCALE: f32 = 0.04 / 0.07;
/// The most chroma a tint gives an icon: the app icons' palette cap (design/08 section 2.10).
pub const TINT_CHROMA_MAX: f32 = 0.07;

impl Tint {
    /// `icons.monochrome_tint = neutral`: no hue at all.
    pub const NEUTRAL: Tint = Tint {
        hue: 0.0,
        chroma: 0.0,
    };

    /// `icons.monochrome_tint = space`: the hue and chroma of the accent the Space derives
    /// (its first dot at Postmark's weight, design/03-COLOR.md section 5), so the icons follow
    /// the frame. Capped at [`TINT_CHROMA_MAX`].
    pub fn space(dots: &[Dot]) -> Tint {
        Tint::from_hex(&derive(dots, Scheme::Light).accent).unwrap_or(Tint::NEUTRAL)
    }

    /// `icons.monochrome_tint = accent`, or any `#RRGGBB`: its hue and chroma, capped. `None`
    /// when the text is not a hex colour.
    pub fn from_hex(hex: &str) -> Option<Tint> {
        let digits = hex.strip_prefix('#')?;
        if digits.len() != 6 {
            return None;
        }
        let v = u32::from_str_radix(digits, 16).ok()?;
        let lab = oklab([(v >> 16) as u8, (v >> 8) as u8, v as u8]);
        Some(Tint {
            hue: lab[2].atan2(lab[1]).to_degrees().rem_euclid(360.0) as f32,
            chroma: (lab[1].hypot(lab[2]) as f32).min(TINT_CHROMA_MAX),
        })
    }
}

/// Re-colours `pixels` (RGBA8, straight alpha) in place for `style`. A trailing partial pixel
/// is left alone.
pub fn retint(pixels: &mut [u8], style: IconStyle, tint: Tint) {
    if style == IconStyle::Colour {
        return;
    }
    for px in pixels.as_chunks_mut::<4>().0 {
        if px[3] == 0 {
            continue;
        }
        let [l, a, b] = oklab([px[0], px[1], px[2]]);
        let (chroma, hue) = match style {
            IconStyle::Muted => (a.hypot(b) * f64::from(MUTED_SCALE), b.atan2(a)),
            _ => {
                let shape = 1.0 - (2.0 * l - 1.0).powi(2);
                let cover = f64::from(px[3]) / 255.0;
                (
                    f64::from(tint.chroma) * shape.max(0.0) * cover,
                    f64::from(tint.hue).to_radians(),
                )
            }
        };
        let rgb = in_gamut(l, chroma, hue);
        px[..3].copy_from_slice(&rgb);
    }
}

/// The sRGB bytes of an OKLCh colour, chroma lowered until it fits.
fn in_gamut(l: f64, mut chroma: f64, hue: f64) -> [u8; 3] {
    loop {
        let rgb = linear_rgb([l, chroma * hue.cos(), chroma * hue.sin()]);
        if rgb.iter().all(|c| (-1e-6..=1.0 + 1e-6).contains(c)) || chroma <= 0.0 {
            return rgb.map(encode);
        }
        chroma = (chroma - 0.004).max(0.0);
    }
}

fn linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn encode(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let e = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (e * 255.0).round() as u8
}

/// OKLab (Björn Ottosson, 2020) of sRGB bytes: `[L, a, b]`.
fn oklab([r, g, b]: [u8; 3]) -> [f64; 3] {
    let [r, g, b] = [r, g, b].map(linear);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// Linear sRGB of OKLab, unclamped.
fn linear_rgb([l, a, b]: [f64; 3]) -> [f64; 3] {
    let l_ = (l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m_ = (l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s_ = (l - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    [
        4.076_741_662_1 * l_ - 3.307_711_591_3 * m_ + 0.230_969_929_2 * s_,
        -1.268_438_004_6 * l_ + 2.609_757_401_1 * m_ - 0.341_319_396_5 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_614_7 * m_ + 1.707_614_701_0 * s_,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::PRESETS;

    fn lch(px: &[u8]) -> (f64, f64, f64) {
        let [l, a, b] = oklab([px[0], px[1], px[2]]);
        (l, a.hypot(b), b.atan2(a).to_degrees().rem_euclid(360.0))
    }

    const RED: [u8; 4] = [0xE8, 0x48, 0x3C, 255];
    const GREY: [u8; 4] = [128, 128, 128, 255];

    #[test]
    fn colour_leaves_pixels_alone() {
        let mut px = RED.to_vec();
        retint(&mut px, IconStyle::Colour, Tint::NEUTRAL);
        assert_eq!(px, RED);
    }

    /// Muted keeps hue and lightness and scales chroma by 4/7.
    #[test]
    fn muted_scales_chroma() {
        let mut px = RED.to_vec();
        retint(&mut px, IconStyle::Muted, Tint::NEUTRAL);
        let ((l0, c0, h0), (l1, c1, h1)) = (lch(&RED), lch(&px));
        assert!((l0 - l1).abs() < 0.01, "lightness kept");
        assert!((h0 - h1).abs() < 2.0, "hue kept");
        assert!(
            (c1 / c0 - f64::from(MUTED_SCALE)).abs() < 0.03,
            "{c0} -> {c1}"
        );
    }

    /// Monochrome: lightness and alpha kept, the tint's hue, and a grey pixel gains chroma;
    /// neutral makes everything grey.
    #[test]
    fn monochrome_takes_the_tints_hue() {
        let tint = Tint {
            hue: 150.0,
            chroma: 0.06,
        };
        let mut px = [RED, GREY].concat();
        retint(&mut px, IconStyle::Monochrome, tint);
        for (i, before) in [RED, GREY].iter().enumerate() {
            let after = &px[i * 4..i * 4 + 4];
            let ((l0, _, _), (l1, c1, h1)) = (lch(before), lch(after));
            assert!((l0 - l1).abs() < 0.01, "{i}: lightness kept");
            assert!(
                (h1 - 150.0).abs() < 3.0 && c1 > 0.03,
                "{i}: tinted {c1} {h1}"
            );
            assert_eq!(after[3], 255, "alpha kept");
        }
        let mut grey = RED.to_vec();
        retint(&mut grey, IconStyle::Monochrome, Tint::NEUTRAL);
        assert!(lch(&grey).1 < 0.005, "neutral is grey");
    }

    #[test]
    fn transparent_pixels_and_white_stay() {
        let mut px = vec![10, 200, 30, 0, 255, 255, 255, 255];
        retint(
            &mut px,
            IconStyle::Monochrome,
            Tint {
                hue: 20.0,
                chroma: 0.07,
            },
        );
        assert_eq!(&px[..4], &[10, 200, 30, 0]);
        assert!(
            px[4..7].iter().all(|c| *c >= 250),
            "white has no room for chroma"
        );
    }

    /// The Work and Home presets tint near their own hues, at most the palette's cap.
    #[test]
    fn space_tints_follow_the_preset() {
        for (preset, hue) in [(0, 268.0_f32), (1, 152.0)] {
            let t = Tint::space(PRESETS[preset].dots);
            let d = (t.hue - hue + 540.0).rem_euclid(360.0) - 180.0;
            assert!(d.abs() < 12.0, "preset {preset}: {}", t.hue);
            assert!(
                t.chroma > 0.03 && t.chroma <= TINT_CHROMA_MAX,
                "{}",
                t.chroma
            );
        }
        assert_eq!(Tint::from_hex("nope"), None);
        assert_eq!(Tint::from_hex("#12345"), None);
    }
}
