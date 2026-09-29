//! OKLab and OKLCh (Björn Ottosson, 2020, with his published matrices), and how far apart two
//! colours look.

use super::srgb::{LinearRgb, Srgb};

/// A colour in OKLab: lightness `l` (0..=1) and the opponent axes `a` (green-red) and `b`
/// (blue-yellow).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// A colour in OKLCh: OKLab's lightness, its chroma and its hue in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    /// The hue angle in radians.
    pub h: f64,
}

impl Oklch {
    /// `oklch(l c hue)` with the hue in degrees, turned to radians as the CSS prototype does
    /// (`hue * PI / 180`).
    pub fn from_degrees(l: f64, c: f64, degrees: f64) -> Oklch {
        Oklch {
            l,
            c,
            h: degrees * std::f64::consts::PI / 180.0,
        }
    }
}

impl Oklab {
    /// The Euclidean distance between two colours (0 is identical; about 0.02 is a
    /// just-noticeable step).
    pub fn distance(self, other: Oklab) -> f64 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }
}

impl From<LinearRgb> for Oklab {
    fn from(LinearRgb([r, g, b]): LinearRgb) -> Self {
        let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
        let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
        let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
        Oklab {
            l: 0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
            a: 1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
            b: 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
        }
    }
}

impl From<Srgb> for Oklab {
    fn from(srgb: Srgb) -> Self {
        Oklab::from(LinearRgb::from(srgb))
    }
}

impl From<Oklab> for LinearRgb {
    /// Unclamped: the caller decides how to bring an out-of-gamut colour in.
    fn from(Oklab { l, a, b }: Oklab) -> Self {
        let l3 = (l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
        let m3 = (l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
        let s3 = (l - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
        LinearRgb([
            4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
            -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
            -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
        ])
    }
}

impl From<Oklab> for Srgb {
    /// Clamped into the gamut and rounded to the nearest byte.
    fn from(lab: Oklab) -> Self {
        Srgb::from(LinearRgb::from(lab))
    }
}

impl From<Oklab> for Oklch {
    fn from(Oklab { l, a, b }: Oklab) -> Self {
        Oklch {
            l,
            c: a.hypot(b),
            h: b.atan2(a),
        }
    }
}

impl From<Oklch> for Oklab {
    fn from(Oklch { l, c, h }: Oklch) -> Self {
        Oklab {
            l,
            a: c * h.cos(),
            b: c * h.sin(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Oklab, Oklch};
    use crate::core::colour::srgb::Srgb;

    #[test]
    fn every_grey_has_no_chroma_and_primaries_their_own() {
        // Ottosson's published OKLCh chroma for the sRGB primaries.
        #[rustfmt::skip]
        const CASES: &[([u8; 3], f64)] = &[
            ([0, 0, 0], 0.0),
            ([255, 255, 255], 0.0),
            ([119, 119, 119], 0.0),
            ([255, 0, 0], 0.2577),
            ([0, 255, 0], 0.2948),
            ([0, 0, 255], 0.3132),
        ];
        for &(rgb, want) in CASES {
            let got = Oklch::from(Oklab::from(Srgb(rgb))).c;
            assert!((got - want).abs() < 0.001, "{rgb:?}: {got} vs {want}");
        }
    }

    #[test]
    fn bytes_round_trip_through_oklab() {
        for rgb in [
            [248, 249, 246],
            [255, 255, 255],
            [21, 24, 20],
            [42, 47, 40],
            [232, 72, 60],
        ] {
            assert_eq!(Srgb::from(Oklab::from(Srgb(rgb))), Srgb(rgb));
        }
    }
}
