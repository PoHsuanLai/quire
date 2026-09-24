//! The safe triangle (design/13 section 13.5): `q` is shielded iff it lies inside the triangle
//! `(from, top - (0, 4), bottom + (0, 4))`; a barycentric sign test.

use super::types::SafeTriangle;
use crate::geometry::{Point, Px};

/// How far the submenu's corners are pushed out vertically: the triangle's hysteresis.
const INFLATE: Px = Px(4.0);

/// Whether `q` is inside (or on an edge of) the triangle `a b c`, either winding.
pub fn inside(q: Point, a: Point, b: Point, c: Point) -> bool {
    let cross = |p: Point, r: Point, s: Point| {
        let (px, py) = (f64::from(p.x.0), f64::from(p.y.0));
        (f64::from(r.x.0) - px) * (f64::from(s.y.0) - py)
            - (f64::from(r.y.0) - py) * (f64::from(s.x.0) - px)
    };
    let sides = [cross(a, b, q), cross(b, c, q), cross(c, a, q)];
    let negative = sides.iter().any(|&d| d < 0.0);
    let positive = sides.iter().any(|&d| d > 0.0);
    !(negative && positive)
}

/// Whether `q` is inside `guard`'s triangle, the submenu's corners inflated 4 px.
pub fn shielded(guard: &SafeTriangle, q: Point) -> bool {
    let top = Point {
        x: guard.top.x,
        y: guard.top.y - INFLATE,
    };
    let bottom = Point {
        x: guard.bottom.x,
        y: guard.bottom.y + INFLATE,
    };
    inside(q, guard.from, top, bottom)
}
