use image::{Rgba, Rgba32FImage};

use crate::{Plane, PlateGrid, Srgb8, Stops, Template, gradient, plate::superellipse, plate_mask};

/// Straight-alpha "over" in encoded sRGB (how CSS composites).
pub fn over(top: [f32; 4], under: [f32; 4]) -> [f32; 4] {
    let a = top[3] + under[3] * (1.0 - top[3]);
    if a <= 0.0 {
        return [0.0; 4];
    }
    let c = |i: usize| (top[i] * top[3] + under[i] * under[3] * (1.0 - top[3])) / a;
    [c(0), c(1), c(2), a]
}

/// `top` over `under`, pixel by pixel (same size).
pub fn over_image(top: &Rgba32FImage, under: &Rgba32FImage) -> Rgba32FImage {
    Rgba32FImage::from_fn(top.width(), top.height(), |x, y| {
        Rgba(over(top.get_pixel(x, y).0, under.get_pixel(x, y).0))
    })
}

/// A flat colour layer whose alpha is `coverage * opacity`.
fn tint(coverage: &Plane, colour: [f32; 3], opacity: f32) -> Rgba32FImage {
    Rgba32FImage::from_fn(coverage.width, coverage.height, |x, y| {
        Rgba([
            colour[0],
            colour[1],
            colour[2],
            coverage.at(i64::from(x), i64::from(y)) * opacity,
        ])
    })
}

/// 08 2.5 scales its 48 px values by `plate side / 48`; never below one pixel.
fn unit(grid: PlateGrid) -> f32 {
    (grid.side as f32 / 48.0).max(1.0)
}

/// The shadowless icon (08 2.6 `flat`): gradient plate, the fitted object, the inner top
/// highlight (sizes >= 24) and the inner rim, all clipped to the squircle.
pub fn compose(grid: PlateGrid, stops: Stops, object: &Rgba32FImage, t: &Template) -> Rgba32FImage {
    let mask = plate_mask(grid, t);
    let s = unit(grid);
    let step = s.round() as i64;
    let highlight = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
        let (x, y) = (i64::from(x), i64::from(y));
        mask.at(x, y) * (1.0 - mask.at(x, y - step))
    });
    let inner = superellipse(
        grid.canvas,
        (grid.centre(), grid.centre()),
        grid.half() - s,
        t.exponent,
    );
    let rim = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
        let (x, y) = (i64::from(x), i64::from(y));
        mask.at(x, y) * (1.0 - inner.at(x, y))
    });
    let plate = gradient(grid, stops);
    let base = over_image(object, &plate);
    let base = over_image(&tint(&rim, [0.0; 3], 0.08), &base);
    let base = match grid.canvas {
        c if c >= 24 => over_image(&tint(&highlight, [1.0; 3], 0.35), &base),
        _ => base,
    };
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let p = base.get_pixel(x, y).0;
        Rgba([p[0], p[1], p[2], p[3] * mask.at(i64::from(x), i64::from(y))])
    })
}

/// Whether an export carries the baked drop shadow (08 2.5: only hicolor files >= 48 px).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shadow {
    Baked,
    Omitted,
}

/// `flat` over the `--shadow-2` drop shadow (08 2.5: `0 6px 16px -6px rgba(26,30,26,.30)` at
/// 48 px, scaled). The shadow is a squircle shrunk by the spread, moved down by the offset
/// and blurred with sigma = blur / 2; whatever falls past the canvas edge is lost.
pub fn drop_shadow(flat: &Rgba32FImage, grid: PlateGrid, t: &Template) -> Rgba32FImage {
    let s = unit(grid);
    let shape = superellipse(
        grid.canvas,
        (grid.centre(), grid.centre() + 6.0 * s),
        grid.half() - 6.0 * s,
        t.exponent,
    );
    let shadow = tint(&shape.blurred(8.0 * s), Srgb8::hex(0x1A1E1A).unit(), 0.30);
    over_image(flat, &shadow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Family;

    /// (name, top, under, expected).
    type OverCase = (&'static str, [f32; 4], [f32; 4], [f32; 4]);

    #[test]
    fn over_cases() {
        const CASES: &[OverCase] = &[
            (
                "opaque top wins",
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ),
            (
                "clear top shows under",
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ),
            (
                "half over opaque mixes",
                [1.0, 1.0, 1.0, 0.5],
                [0.0, 0.0, 0.0, 1.0],
                [0.5, 0.5, 0.5, 1.0],
            ),
            ("both clear", [0.0; 4], [0.0; 4], [0.0; 4]),
        ];
        for (name, top, under, want) in CASES {
            let got = over(*top, *under);
            assert!(
                got.iter().zip(want).all(|(g, w)| (g - w).abs() < 1e-6),
                "{name}: {got:?}"
            );
        }
    }

    /// The plate is transparent outside the squircle, carries the highlight on its top edge
    /// and the gradient in its middle; the shadow darkens only below the plate.
    #[test]
    fn composed_plate_layers() {
        let t = Template::default();
        let grid = PlateGrid::for_canvas(256, &t);
        let flat = compose(grid, Family::Blue.stops(), &Rgba32FImage::new(256, 256), &t);
        assert_eq!(flat.get_pixel(2, 2).0[3], 0.0, "outside the plate");
        let top = flat.get_pixel(128, grid.origin + 1).0;
        let bare = gradient(grid, Family::Blue.stops())
            .get_pixel(128, grid.origin + 1)
            .0;
        assert!(
            top[0] > bare[0] + 0.1,
            "top edge is highlighted: {top:?} vs {bare:?}"
        );
        let master = drop_shadow(&flat, grid, &t);
        assert!(
            master.get_pixel(128, 254).0[3] > 0.0,
            "shadow below the plate"
        );
        assert!(
            master.get_pixel(128, 1).0[3] < 0.01,
            "almost no shadow above the plate"
        );
    }
}
