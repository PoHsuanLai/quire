//! The abstract vocabulary (design/08-ICONS.md 2.8): a handful of filled shapes in the 24-unit
//! glyph grid, each a signed distance (negative inside) so every edge is anti-aliased
//! analytically at any size. Nothing is outlined: a chevron or a bar is a filled, round-ended
//! band, and form comes from the emboss (emblem.rs), not from strokes.

use serde::Deserialize;

/// The default width of a chevron or bar band: 2 units of the 24 grid, the glyph weight (08 1.2).
pub const BAND: f32 = 2.0;

fn band() -> f32 {
    BAND
}

/// A point in the 24-unit grid laid over the plate (0,0 top-left, 24,24 bottom-right).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Pt(pub f32, pub f32);

/// Corner radii, clockwise from top-left. Post's card corner is `[3, 3, 3, 1]` in this grid
/// (07-LOOKS 3.2: `12px 12px 12px 4px`).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Corners(pub [f32; 4]);

/// Which corner a fold turns down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// Which way a chevron points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Heading {
    Right,
    Down,
}

/// One shape of the vocabulary; every shape is a filled area.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Shape {
    /// A rounded rectangle.
    Rect {
        at: Pt,
        size: Pt,
        corners: Corners,
    },
    Circle {
        centre: Pt,
        radius: f32,
    },
    /// A `>` (or `v`) of two round-ended bands meeting at `tip`, arms `reach` long on each axis.
    Chevron {
        tip: Pt,
        reach: f32,
        heading: Heading,
        #[serde(default = "band")]
        width: f32,
    },
    /// A straight band with round ends.
    Bar {
        from: Pt,
        to: Pt,
        #[serde(default = "band")]
        width: f32,
    },
    /// The turned-down triangle of a folded corner: the right angle sits at `corner` of the
    /// square of side `size` whose opposite corner is `at`.
    Fold {
        at: Pt,
        size: f32,
        corner: Corner,
    },
    /// Any convex or concave polygon, corners rounded by `round`.
    Polygon {
        points: Vec<Pt>,
        #[serde(default)]
        round: f32,
    },
}

fn sub(a: Pt, b: Pt) -> Pt {
    Pt(a.0 - b.0, a.1 - b.1)
}

fn dot(a: Pt, b: Pt) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn len(a: Pt) -> f32 {
    dot(a, a).sqrt()
}

/// Distance from `p` to the segment `a`-`b`.
fn segment(p: Pt, a: Pt, b: Pt) -> f32 {
    let (pa, ba) = (sub(p, a), sub(b, a));
    let h = (dot(pa, ba) / dot(ba, ba).max(1e-9)).clamp(0.0, 1.0);
    len(Pt(pa.0 - ba.0 * h, pa.1 - ba.1 * h))
}

/// Signed distance to a rounded box (Quilez), radii picked by quadrant.
fn rect(p: Pt, at: Pt, size: Pt, corners: Corners) -> f32 {
    let c = Pt(at.0 + size.0 / 2.0, at.1 + size.1 / 2.0);
    let q = sub(p, c);
    let [tl, tr, br, bl] = corners.0;
    let r = match (q.0 >= 0.0, q.1 >= 0.0) {
        (false, false) => tl,
        (true, false) => tr,
        (true, true) => br,
        (false, true) => bl,
    };
    let r = r.min(size.0 / 2.0).min(size.1 / 2.0);
    let d = Pt(q.0.abs() - size.0 / 2.0 + r, q.1.abs() - size.1 / 2.0 + r);
    len(Pt(d.0.max(0.0), d.1.max(0.0))) + d.0.max(d.1).min(0.0) - r
}

/// Signed distance to a polygon (Quilez's winding-number form).
fn polygon(p: Pt, v: &[Pt]) -> f32 {
    let n = v.len();
    let first = sub(p, v[0]);
    let (mut d, mut s) = (dot(first, first), 1.0_f32);
    for i in 0..n {
        let (a, b) = (v[i], v[(i + n - 1) % n]);
        let (e, w) = (sub(b, a), sub(p, a));
        let t = (dot(w, e) / dot(e, e).max(1e-9)).clamp(0.0, 1.0);
        let proj = Pt(w.0 - e.0 * t, w.1 - e.1 * t);
        d = d.min(dot(proj, proj));
        let c = [p.1 >= a.1, p.1 < b.1, e.0 * w.1 > e.1 * w.0];
        if c.iter().all(|x| *x) || c.iter().all(|x| !*x) {
            s = -s;
        }
    }
    s * d.sqrt()
}

fn fold_points(at: Pt, size: f32, corner: Corner) -> [Pt; 3] {
    let (x0, y0, x1, y1) = (at.0, at.1, at.0 + size, at.1 + size);
    match corner {
        Corner::TopLeft => [Pt(x0, y1), Pt(x0, y0), Pt(x1, y0)],
        Corner::TopRight => [Pt(x0, y0), Pt(x1, y0), Pt(x1, y1)],
        Corner::BottomRight => [Pt(x1, y0), Pt(x1, y1), Pt(x0, y1)],
        Corner::BottomLeft => [Pt(x0, y0), Pt(x0, y1), Pt(x1, y1)],
    }
}

/// The shape's signed distance at `p` in grid units, negative inside.
pub fn distance(shape: &Shape, p: Pt) -> f32 {
    match shape {
        Shape::Rect { at, size, corners } => rect(p, *at, *size, *corners),
        Shape::Circle { centre, radius } => len(sub(p, *centre)) - radius,
        Shape::Chevron {
            tip,
            reach,
            heading,
            width,
        } => {
            let (a, b) = match heading {
                Heading::Right => (
                    Pt(tip.0 - reach, tip.1 - reach),
                    Pt(tip.0 - reach, tip.1 + reach),
                ),
                Heading::Down => (
                    Pt(tip.0 - reach, tip.1 - reach),
                    Pt(tip.0 + reach, tip.1 - reach),
                ),
            };
            segment(p, a, *tip).min(segment(p, *tip, b)) - width / 2.0
        }
        Shape::Bar { from, to, width } => segment(p, *from, *to) - width / 2.0,
        Shape::Fold { at, size, corner } => polygon(p, &fold_points(*at, *size, *corner)),
        Shape::Polygon { points, round } => polygon(p, points) - round,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: Shape = Shape::Rect {
        at: Pt(4.0, 4.0),
        size: Pt(16.0, 16.0),
        corners: Corners([3.0, 3.0, 3.0, 1.0]),
    };

    /// (name, shape, point, expected distance).
    #[test]
    fn distances() {
        let tri = Shape::Polygon {
            points: vec![Pt(0.0, 0.0), Pt(10.0, 0.0), Pt(0.0, 10.0)],
            round: 0.0,
        };
        let cases: &[(&str, &Shape, Pt, f32)] = &[
            ("rect centre is 8 inside", &SQUARE, Pt(12.0, 12.0), -8.0),
            ("rect edge", &SQUARE, Pt(12.0, 4.0), 0.0),
            ("rect outside", &SQUARE, Pt(12.0, 2.0), 2.0),
            (
                "radius 1 keeps a point near its corner",
                &SQUARE,
                Pt(4.5, 19.5),
                -0.2929,
            ),
            (
                "radius 3 cuts the same point",
                &SQUARE,
                Pt(4.5, 4.5),
                0.5355,
            ),
            ("triangle inside", &tri, Pt(1.0, 1.0), -1.0),
            ("triangle outside", &tri, Pt(-2.0, 5.0), 2.0),
        ];
        for (name, shape, p, want) in cases {
            let got = distance(shape, *p);
            assert!((got - want).abs() < 1e-3, "{name}: {got}");
        }
    }

    #[test]
    fn chevron_is_a_filled_band() {
        let c = Shape::Chevron {
            tip: Pt(12.0, 12.0),
            reach: 4.0,
            heading: Heading::Right,
            width: BAND,
        };
        assert!(
            (distance(&c, Pt(12.0, 12.0)) + 1.0).abs() < 1e-6,
            "tip is 1 inside"
        );
        assert!(
            (distance(&c, Pt(8.0, 8.0)) + 1.0).abs() < 1e-6,
            "arm end is 1 inside"
        );
        assert!(
            (distance(&c, Pt(14.0, 12.0)) - 1.0).abs() < 1e-3,
            "edge 1 past the tip"
        );
    }
}
