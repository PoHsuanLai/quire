//! kurbo geometry to krilla's: an affine to a transform, a shape to a path.

use krilla::geom::{Path, PathBuilder, Transform};
use peniko::kurbo::{Affine, PathEl, Shape};

/// How closely curves that are not already Béziers (circles, rounded rects) are approximated,
/// in the shape's own units: a tenth of a CSS pixel is invisible on paper.
const TOLERANCE: f64 = 0.1;

/// `affine` as krilla's row-major transform.
pub(crate) fn transform(affine: Affine) -> Transform {
    let [a, b, c, d, e, f] = affine.as_coeffs();
    Transform::from_row(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32)
}

/// `shape` with `affine` applied to every point, as a krilla path; `None` when it is empty
/// (krilla cannot hold an empty path, and there is nothing to draw).
pub(crate) fn path(shape: &impl Shape, affine: Affine) -> Option<Path> {
    let mut builder = PathBuilder::new();
    for element in shape.path_elements(TOLERANCE) {
        match affine * element {
            PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
            PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
            PathEl::QuadTo(c, p) => builder.quad_to(c.x as f32, c.y as f32, p.x as f32, p.y as f32),
            PathEl::CurveTo(c1, c2, p) => builder.cubic_to(
                c1.x as f32,
                c1.y as f32,
                c2.x as f32,
                c2.y as f32,
                p.x as f32,
                p.y as f32,
            ),
            PathEl::ClosePath => builder.close(),
        }
    }
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::{path, transform};
    use peniko::kurbo::{Affine, BezPath, Rect};

    #[test]
    fn an_empty_shape_has_no_path() {
        assert!(path(&BezPath::new(), Affine::IDENTITY).is_none());
    }

    #[test]
    fn a_rect_has_a_path() {
        assert!(
            path(
                &Rect::new(0.0, 0.0, 10.0, 5.0),
                Affine::translate((3.0, 4.0))
            )
            .is_some()
        );
    }

    #[test]
    fn coefficients_keep_their_order() {
        let t = transform(Affine::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        assert_eq!(
            (t.sx(), t.ky(), t.kx(), t.sy(), t.tx(), t.ty()),
            (1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
        );
    }
}
