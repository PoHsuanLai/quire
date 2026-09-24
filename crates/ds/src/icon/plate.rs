//! The continuous-curvature squircle (design/08-ICONS.md sections 2.1 and 4.1): the Lamé
//! superellipse `|x/a|^n + |y/a|^n = 1` with `n = 5`. One module owns the shape: the app-icon
//! plate is the whole superellipse, and a [`crate::Corner::Squircle`] corner is one quadrant of
//! it spanning `2 r` along each edge, joined to the straight sides. At `n > 2` the curve's
//! curvature falls to zero where it meets the axis, so the join is continuous in curvature, which
//! a CSS `border-radius` (a circle) is not.
//!
//! Blitz paints `mask-image` with SVG data URLs (spike S7) and composites several mask layers
//! (`mask-composite: add`), so a squircle is drawn as a mask: four quadrant images at the
//! corners and two rectangles for the cross between them. The shapes are sampled here, as data.

use super::external::IconUrl;
use std::f64::consts::FRAC_PI_2;

/// The superellipse exponent (design/08 section 2.1, proposed; settled for the shell
/// 2026-09-24).
pub const EXPONENT: f64 = 5.0;

/// How far a squircle corner of nominal radius `r` reaches along each edge: `2 r`.
pub const EXTENT_PER_RADIUS: f64 = 2.0;

/// The circular radius that touches the squircle corner at its 45 degree point, as a share of the
/// nominal radius: `2 (1 - 2^(-1/5)) / (1 - 1/sqrt 2)`, about .884. The squircle lies inside this
/// circle everywhere and at most .03 r from it, so a squircle element's shadows and hairlines
/// are drawn from it (Blitz draws a `box-shadow` from the `border-radius`, never the mask).
pub fn shadow_radius_share() -> f64 {
    let diagonal = EXTENT_PER_RADIUS * (1.0 - 2f64.powf(-1.0 / EXPONENT));
    diagonal / (1.0 - std::f64::consts::FRAC_1_SQRT_2)
}

/// Samples per quadrant for a corner mask, and for each quadrant of the plate.
const SAMPLES: u32 = 48;

/// One of the four corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quadrant {
    /// Top left.
    TopLeft,
    /// Top right.
    TopRight,
    /// Bottom left.
    BottomLeft,
    /// Bottom right.
    BottomRight,
}

impl Quadrant {
    /// In mask-layer order.
    pub const ALL: [Quadrant; 4] = [
        Quadrant::TopLeft,
        Quadrant::TopRight,
        Quadrant::BottomLeft,
        Quadrant::BottomRight,
    ];

    /// The `mask-position` keywords.
    pub fn position(self) -> &'static str {
        match self {
            Quadrant::TopLeft => "left top",
            Quadrant::TopRight => "right top",
            Quadrant::BottomLeft => "left bottom",
            Quadrant::BottomRight => "right bottom",
        }
    }

    /// Mirrors a top-left point in a 100 x 100 box into this quadrant.
    fn mirror(self, (x, y): (f64, f64)) -> (f64, f64) {
        match self {
            Quadrant::TopLeft => (x, y),
            Quadrant::TopRight => (100.0 - x, y),
            Quadrant::BottomLeft => (x, 100.0 - y),
            Quadrant::BottomRight => (100.0 - x, 100.0 - y),
        }
    }

    /// The corner of the box the shape fills to (the inner corner).
    fn inner(self) -> (f64, f64) {
        self.mirror((100.0, 100.0))
    }
}

/// The point of the superellipse of semi-axis `a` centred on `(a, a)` at parameter `t` in the
/// top-left quadrant (`t = 0` on the left edge's middle, `t = pi/2` on the top edge's).
pub fn point(a: f64, t: f64) -> (f64, f64) {
    let power = 2.0 / EXPONENT;
    (a - a * t.cos().powf(power), a - a * t.sin().powf(power))
}

/// The quadrant's outline in a 100 x 100 box: from the left (or right) edge's end of the curve
/// round to the top (or bottom) edge's, then the inner corner.
fn quadrant_points(quadrant: Quadrant) -> Vec<(f64, f64)> {
    let curve = (0..=SAMPLES).map(|i| {
        let t = FRAC_PI_2 * f64::from(i) / f64::from(SAMPLES);
        quadrant.mirror(point(100.0, t))
    });
    curve.chain([quadrant.inner()]).collect()
}

/// A closed polygon as SVG path data, to two decimals.
fn path(points: &[(f64, f64)]) -> String {
    let steps = points
        .iter()
        .enumerate()
        .map(|(i, (x, y))| format!("{}{x:.2} {y:.2}", if i == 0 { "M" } else { "L" }))
        .collect::<String>();
    format!("{steps}Z")
}

/// An SVG document filling `d` in the mask's opaque colour, stretched to whatever box it masks.
fn document(d: &str) -> String {
    format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100' \
         preserveAspectRatio='none'><path d='{d}'/></svg>"
    )
}

/// The mask image of one corner quadrant.
pub fn quadrant_mask(quadrant: Quadrant) -> IconUrl {
    IconUrl::svg(&document(&path(&quadrant_points(quadrant))))
}

/// A full, opaque rectangle: the cross between the four corners.
pub fn fill_mask() -> IconUrl {
    IconUrl::svg(&document("M0 0H100V100H0Z"))
}

/// The whole superellipse in a 100 x 100 box: the app-icon plate.
pub fn plate_mask() -> IconUrl {
    let a = 50.0;
    let points: Vec<(f64, f64)> = (0..SAMPLES * 4)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * f64::from(i) / f64::from(SAMPLES * 4);
            let power = 2.0 / EXPONENT;
            let x = a + a * t.cos().signum() * t.cos().abs().powf(power);
            let y = a + a * t.sin().signum() * t.sin().abs().powf(power);
            (x, y)
        })
        .collect();
    IconUrl::svg(&document(&path(&points)))
}

/// Whether `(x, y)`, measured from a corner of a squircle corner of extent `k`, lies inside the
/// shape: the analytic test the pixel proofs compare against.
pub fn inside_corner(k: f64, x: f64, y: f64) -> bool {
    if x >= k || y >= k {
        return true;
    }
    let u = (k - x) / k;
    let v = (k - y) / k;
    u.powf(EXPONENT) + v.powf(EXPONENT) <= 1.0
}

#[cfg(test)]
mod tests {
    use super::{
        EXPONENT, Quadrant, fill_mask, inside_corner, plate_mask, point, quadrant_mask,
        shadow_radius_share,
    };

    #[test]
    fn every_sample_lies_on_the_superellipse() {
        for i in 0..=16 {
            let t = std::f64::consts::FRAC_PI_2 * f64::from(i) / 16.0;
            let (x, y) = point(1.0, t);
            let sum = (1.0 - x).abs().powf(EXPONENT) + (1.0 - y).abs().powf(EXPONENT);
            assert!((sum - 1.0).abs() < 1e-9, "t {t}: {sum}");
        }
    }

    #[test]
    fn the_corner_is_fuller_than_a_circle_of_the_same_radius_at_45_degrees() {
        // Nominal radius 1: the squircle spans 2. Its 45 degree point sits .366 in from the
        // corner along the diagonal; a circle of radius 1 sits .414 in.
        let (x, _) = point(2.0, std::f64::consts::FRAC_PI_4);
        let squircle = x * std::f64::consts::SQRT_2;
        let circle = std::f64::consts::SQRT_2 - 1.0;
        assert!((squircle - 0.366).abs() < 0.001, "{squircle}");
        assert!(circle - squircle > 0.04);
        assert!((shadow_radius_share() - 0.884).abs() < 0.001);
        assert!(inside_corner(2.0, 0.3, 0.3));
        assert!(!inside_corner(2.0, 0.2, 0.2));
    }

    #[test]
    fn the_masks_are_svg_data_urls() {
        for quadrant in Quadrant::ALL {
            let url = quadrant_mask(quadrant);
            assert!(url.as_str().starts_with("data:image/svg+xml,"));
            assert!(!url.as_str().contains('"'));
        }
        assert!(fill_mask().as_str().contains("H100V100H0Z"));
        assert!(plate_mask().as_str().len() > 1000);
    }
}
