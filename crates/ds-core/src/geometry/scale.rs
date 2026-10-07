//! The device scale: how many device pixels one logical pixel covers.
//!
//! At a fractional scale (1.25, 1.5, 1.75) a one logical pixel line covers a fractional number of
//! device pixels and blurs into two half-alpha rows; the pixel tokens (`pixel`)
//! and the host's layout snap (`ds_blitz::snap`) are computed from this value so it does not.

/// Device pixels per logical pixel, in 120ths: `Scale(120)` is 1x, `Scale(180)` is 1.5x.
///
/// 120ths because that is the wire unit of `wp_fractional_scale_v1.preferred_scale`, and the
/// `Scale(u32)` shell-host already names: a host passes its value through unchanged, and every
/// scale a compositor can announce is exact here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Scale(pub u32);

/// Whether device pixels fall on logical pixel boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Grid {
    /// A whole scale (1x, 2x, 3x): every whole logical pixel is a whole number of device pixels,
    /// so Blitz's own rounding to logical pixels already lands on the device grid.
    Whole,
    /// A fractional scale (1.25x, 1.5x, 1.75x): a whole logical pixel can start half-way
    /// through a device pixel, so positions and widths must be snapped to the device grid.
    Fractional,
}

impl Scale {
    /// The denominator: 120ths.
    pub const DENOMINATOR: u32 = 120;

    /// 1x, what a root draws at when no one says otherwise.
    pub const ONE: Scale = Scale(Scale::DENOMINATOR);

    /// A scale given in hundredths (`ds_harness::Viewport::scale_percent`): 150 is `Scale(180)`.
    /// Exact for every multiple of 5 %, which covers every scale a desktop offers.
    pub fn from_percent(percent: u16) -> Scale {
        Scale(u32::from(percent) * Scale::DENOMINATOR / 100)
    }

    /// A scale given as the factor a platform reports (winit's `scale_factor`, 1.5 for 150 %),
    /// rounded to the nearest 120th; `None` for a factor that is not a positive number.
    pub fn from_factor(factor: f64) -> Option<Scale> {
        let valid = factor.is_finite() && factor > 0.0;
        // Bounded by the validity check and `max(1.0)`, so the cast cannot truncate or wrap.
        valid.then(|| Scale((factor * f64::from(Scale::DENOMINATOR)).round().max(1.0) as u32))
    }

    /// The numerator, never zero: a zero scale would make one device pixel infinitely wide.
    pub fn numerator(self) -> u32 {
        self.0.max(1)
    }

    /// The scale as a multiplier, for the one place arithmetic needs it (a renderer's scale
    /// factor). Never stored.
    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator()) / f64::from(Scale::DENOMINATOR)
    }

    /// Whether this scale's device grid coincides with the logical one.
    pub fn grid(self) -> Grid {
        match self.numerator() % Scale::DENOMINATOR {
            0 => Grid::Whole,
            _ => Grid::Fractional,
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale::ONE
    }
}

#[cfg(test)]
mod tests {
    use super::{Grid, Scale};

    const PERCENT: &[(u16, Scale, Grid)] = &[
        (100, Scale(120), Grid::Whole),
        (125, Scale(150), Grid::Fractional),
        (150, Scale(180), Grid::Fractional),
        (175, Scale(210), Grid::Fractional),
        (200, Scale(240), Grid::Whole),
    ];

    #[test]
    fn percent_converts_exactly_to_120ths() {
        for &(percent, scale, grid) in PERCENT {
            assert_eq!(Scale::from_percent(percent), scale, "{percent}");
            assert_eq!(scale.grid(), grid, "{percent}");
        }
    }

    #[test]
    fn a_platform_factor_rounds_to_the_nearest_120th() {
        const CASES: &[(&str, f64, Option<Scale>)] = &[
            ("one", 1.0, Some(Scale(120))),
            ("one and a half", 1.5, Some(Scale(180))),
            ("a third more", 1.3333333, Some(Scale(160))),
            ("below one", 0.5, Some(Scale(60))),
            ("zero", 0.0, None),
            ("negative", -2.0, None),
            ("not a number", f64::NAN, None),
            ("infinite", f64::INFINITY, None),
        ];
        for (name, factor, want) in CASES {
            assert_eq!(Scale::from_factor(*factor), *want, "{name}");
        }
    }

    #[test]
    fn a_zero_scale_is_treated_as_the_smallest_one() {
        assert_eq!(Scale(0).numerator(), 1);
        assert!(Scale(0).as_f64() > 0.0);
    }
}
