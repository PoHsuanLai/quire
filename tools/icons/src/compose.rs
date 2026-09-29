use image::{Rgba, Rgba32FImage};

use crate::{Plane, PlateGrid, Srgb8, Template, plate::superellipse};

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
}
