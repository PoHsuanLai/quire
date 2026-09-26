//! The persona's geometry on a 100 x 100 canvas (design/24-PERSONA.md section 3.1): the head's
//! outline as a superellipse, where the face's features sit on it, and path helpers. Pure: every
//! function takes numbers and returns numbers or path text.

use super::spec::HeadShape;
use std::f64::consts::PI;
use std::fmt::Write as _;

/// A head: a superellipse `|x/rx|^n + |y/ry|^n = 1` centred at `(cx, cy)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Head {
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
    pub n: f64,
}

impl Head {
    /// The outline for `shape`.
    pub(crate) fn of(shape: HeadShape) -> Head {
        let (rx, ry, n, cy) = match shape {
            HeadShape::Round => (29.5, 28.5, 2.0, 56.0),
            HeadShape::Soft => (30.5, 27.5, 3.0, 56.0),
            HeadShape::Tall => (26.5, 30.5, 2.3, 55.0),
            HeadShape::Wide => (33.0, 26.5, 2.5, 57.0),
        };
        Head {
            cx: 50.0,
            cy,
            rx,
            ry,
            n,
        }
    }

    /// The point at angle `t` (radians; `3/2 pi` is the crown), `grow` units outside the outline.
    pub(crate) fn point(self, t: f64, grow: f64) -> (f64, f64) {
        let e = 2.0 / self.n;
        (
            self.cx + (self.rx + grow) * signed_pow(t.cos(), e),
            self.cy + (self.ry + grow) * signed_pow(t.sin(), e),
        )
    }

    /// The whole outline, `grow` units out.
    pub(crate) fn outline(self, grow: f64) -> String {
        let points: Vec<(f64, f64)> = (0..72)
            .map(|step| self.point(f64::from(step) * 2.0 * PI / 72.0, grow))
            .collect();
        polygon(&points)
    }

    /// The angles, left to right over the crown, where the outline `grow` units out is above
    /// `y`: from `t0` to `t1`.
    pub(crate) fn span_above(self, y: f64, grow: f64) -> (f64, f64) {
        let depth = ((self.cy - y) / (self.ry + grow)).clamp(0.0, 1.0);
        let k = depth.powf(self.n / 2.0).clamp(0.0, 1.0);
        (PI + k.asin(), 2.0 * PI - k.asin())
    }

    /// The outline `grow` units out above `y`, left to right, as points.
    pub(crate) fn crown(self, y: f64, grow: f64) -> Vec<(f64, f64)> {
        let (t0, t1) = self.span_above(y, grow);
        (0..=40)
            .map(|step| self.point(t0 + (t1 - t0) * f64::from(step) / 40.0, grow))
            .collect()
    }

    /// The top of the head.
    pub(crate) fn top(self) -> f64 {
        self.cy - self.ry
    }

    /// Where the eyes sit: their height and half their spacing.
    pub(crate) fn eyes(self, wide: Spacing) -> (f64, f64) {
        let share = match wide {
            Spacing::Near => 0.38,
            Spacing::Far => 0.50,
        };
        (self.cy + 0.08 * self.ry, share * self.rx)
    }

    /// A point on the face: `(dx, dy)` in shares of the half widths from the centre.
    pub(crate) fn at(self, dx: f64, dy: f64) -> (f64, f64) {
        (self.cx + dx * self.rx, self.cy + dy * self.ry)
    }
}

/// How far apart the eyes are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spacing {
    /// The usual spacing.
    Near,
    /// Set wide.
    Far,
}

/// `value.abs().powf(e)` with `value`'s sign.
fn signed_pow(value: f64, e: f64) -> f64 {
    value.signum() * value.abs().powf(e)
}

/// A number for path text: one decimal, no `-0.0`.
pub(crate) fn num(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    let rounded = if rounded == 0.0 { 0.0 } else { rounded };
    format!("{rounded}")
}

/// `x y` for path text.
pub(crate) fn pt((x, y): (f64, f64)) -> String {
    format!("{} {}", num(x), num(y))
}

/// A closed polygon through `points`.
pub(crate) fn polygon(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (index, point) in points.iter().enumerate() {
        let verb = if index == 0 { 'M' } else { 'L' };
        let _ = write!(d, "{verb}{}", pt(*point));
    }
    d.push('Z');
    d
}

/// An ellipse as a path.
pub(crate) fn ellipse((cx, cy): (f64, f64), rx: f64, ry: f64) -> String {
    format!(
        "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0Z",
        num(cx - rx),
        num(cy),
        num(rx),
        num(ry),
        num(2.0 * rx),
        num(rx),
        num(ry),
        num(-2.0 * rx)
    )
}

/// A circle as a path.
pub(crate) fn circle(centre: (f64, f64), r: f64) -> String {
    ellipse(centre, r, r)
}

#[cfg(test)]
mod tests {
    use super::{Head, circle, num};
    use crate::components::persona::spec::HeadShape;

    #[test]
    fn numbers_are_short() {
        const CASES: &[(f64, &str)] = &[(1.25, "1.3"), (-0.01, "0"), (50.0, "50"), (-2.26, "-2.3")];
        for &(value, want) in CASES {
            assert_eq!(num(value), want, "{value}");
        }
        assert_eq!(
            circle((50.0, 50.0), 2.0),
            "M48 50a2 2 0 1 0 4 0a2 2 0 1 0 -4 0Z"
        );
    }

    #[test]
    fn every_head_fits_the_disc_with_room_for_ears() {
        for shape in [
            HeadShape::Round,
            HeadShape::Soft,
            HeadShape::Tall,
            HeadShape::Wide,
        ] {
            let head = Head::of(shape);
            assert!(head.top() >= 22.0, "{shape:?} top {}", head.top());
            assert!(head.cy + head.ry <= 88.0, "{shape:?} chin");
            let crown = head.crown(head.cy - 0.4 * head.ry, 0.0);
            let lowest = crown.iter().map(|p| p.1).fold(f64::MIN, f64::max);
            assert!(
                lowest <= head.cy - 0.4 * head.ry + 0.5,
                "{shape:?} crown {lowest}"
            );
        }
    }
}
