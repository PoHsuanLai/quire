/// An 8-bit sRGB colour, as written in the design docs' hex tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Srgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Srgb8 {
    /// `0xRRGGBB` as in the docs' `#RRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Self {
            r: (rgb >> 16) as u8,
            g: (rgb >> 8) as u8,
            b: rgb as u8,
        }
    }

    /// The channels as 0..=1 sRGB-encoded floats.
    pub fn unit(self) -> [f32; 3] {
        [self.r, self.g, self.b].map(|c| f32::from(c) / 255.0)
    }
}

/// A colour in OKLab (Ottosson 2020): perceptual distance is the Euclidean one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

/// sRGB transfer, encoded 0..=1 to linear 0..=1.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB transfer, linear 0..=1 to encoded 0..=1.
pub fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// OKLab of an sRGB-encoded 0..=1 triple.
pub fn oklab(rgb: [f32; 3]) -> Oklab {
    let [r, g, b] = rgb.map(srgb_to_linear);
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let [l, m, s] = [l, m, s].map(f32::cbrt);
    Oklab {
        l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    }
}

fn linear_rgb(c: Oklab) -> [f32; 3] {
    let l = c.l + 0.396_337_78 * c.a + 0.215_803_76 * c.b;
    let m = c.l - 0.105_561_346 * c.a - 0.063_854_17 * c.b;
    let s = c.l - 0.089_484_18 * c.a - 1.291_485_5 * c.b;
    let [l, m, s] = [l, m, s].map(|v| v * v * v);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

/// Whether an OKLab colour lies inside sRGB (so [`srgb`] does not clip it).
pub fn in_srgb(c: Oklab) -> bool {
    linear_rgb(c)
        .iter()
        .all(|v| (-1e-4..=1.0 + 1e-4).contains(v))
}

/// The sRGB-encoded 0..=1 triple of an OKLab colour (clamped to the gamut).
pub fn srgb(c: Oklab) -> [f32; 3] {
    linear_rgb(c).map(linear_to_srgb)
}

/// ΔE_OK: Euclidean distance in OKLab.
pub fn delta_e(p: Oklab, q: Oklab) -> f32 {
    ((p.l - q.l).powi(2) + (p.a - q.a).powi(2) + (p.b - q.b).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: &[(&str, [f32; 3], f32)] = &[
        ("white", [1.0, 1.0, 1.0], 1.0),
        ("black", [0.0, 0.0, 0.0], 0.0),
        ("mid grey", [0.5, 0.5, 0.5], 0.598_1),
    ];

    #[test]
    fn oklab_lightness() {
        for (name, rgb, l) in CASES {
            let got = oklab(*rgb);
            assert!((got.l - l).abs() < 1e-3, "{name}: L {}", got.l);
            assert!(
                got.a.abs() < 1e-3 && got.b.abs() < 1e-3,
                "{name}: grey has chroma"
            );
        }
    }

    #[test]
    fn gamut_check() {
        assert!(in_srgb(oklab([0.2, 0.5, 0.9])));
        let wild = Oklab {
            l: 0.62,
            a: -0.3,
            b: 0.0,
        };
        assert!(!in_srgb(wild));
    }

    #[test]
    fn oklab_round_trips() {
        for rgb in [[0.8, 0.3, 0.1], [0.1, 0.5, 0.9], [0.5, 0.5, 0.5]] {
            let back = srgb(oklab(rgb));
            assert!(
                back.iter().zip(rgb).all(|(b, c)| (b - c).abs() < 1e-3),
                "{rgb:?} {back:?}"
            );
        }
    }

    #[test]
    fn transfer_round_trips() {
        for i in 0..=255u8 {
            let c = f32::from(i) / 255.0;
            assert!((linear_to_srgb(srgb_to_linear(c)) - c).abs() < 1e-5, "{i}");
        }
    }

    #[test]
    fn hex_splits_channels() {
        assert_eq!(
            Srgb8::hex(0xE8483C),
            Srgb8 {
                r: 0xE8,
                g: 0x48,
                b: 0x3C
            }
        );
    }
}
