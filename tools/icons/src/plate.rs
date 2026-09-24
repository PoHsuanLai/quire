use image::{Rgba, Rgba32FImage};

use crate::{Plane, Stops, Template};

/// Where the plate sits on a canvas of a given export size (design/08-ICONS.md 2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlateGrid {
    pub canvas: u32,
    /// Plate side in px.
    pub side: u32,
    /// Offset of the plate's top-left corner on both axes, px.
    pub origin: u32,
}

impl PlateGrid {
    /// 08 2.2: at 48 px and up the plate is `plate` of the canvas, rounded; 32 and 24 keep a
    /// 1 px margin; 16 has a 15 px plate at the origin.
    pub fn for_canvas(canvas: u32, t: &Template) -> Self {
        match canvas {
            16 => Self {
                canvas,
                side: 15,
                origin: 0,
            },
            c if c < 48 => Self {
                canvas,
                side: c - 2,
                origin: 1,
            },
            c => {
                let side = (t.plate.0 * c as f32).round() as u32;
                Self {
                    canvas,
                    side,
                    origin: (c - side) / 2,
                }
            }
        }
    }

    /// Half the plate side: the superellipse's `a`.
    pub fn half(self) -> f32 {
        self.side as f32 / 2.0
    }

    /// The plate centre on both axes.
    pub fn centre(self) -> f32 {
        self.origin as f32 + self.half()
    }
}

/// Coverage of a Lamé superellipse `|x/a|^n + |y/a|^n <= 1` centred at (cx, cy), 4x4
/// supersampled per pixel (08 2.1).
pub fn superellipse(canvas: u32, (cx, cy): (f32, f32), a: f32, n: f32) -> Plane {
    const SUB: u32 = 4;
    Plane::from_fn(canvas, canvas, |x, y| {
        let inside = (0..SUB * SUB)
            .filter(|k| {
                let px = x as f32 + (k % SUB) as f32 / SUB as f32 + 0.5 / SUB as f32;
                let py = y as f32 + (k / SUB) as f32 / SUB as f32 + 0.5 / SUB as f32;
                ((px - cx) / a).abs().powf(n) + ((py - cy) / a).abs().powf(n) <= 1.0
            })
            .count();
        inside as f32 / (SUB * SUB) as f32
    })
}

/// The plate silhouette at one export size: the only function that draws the plate shape.
pub fn plate_mask(grid: PlateGrid, t: &Template) -> Plane {
    superellipse(
        grid.canvas,
        (grid.centre(), grid.centre()),
        grid.half(),
        t.exponent,
    )
}

/// The plate gradient over the whole canvas: `base` at the plate's top-left corner to `deep`
/// at its bottom-right (135deg, 08 2.3), interpolated in encoded sRGB like CSS. Opaque; the
/// caller masks it.
pub fn gradient(grid: PlateGrid, stops: Stops) -> Rgba32FImage {
    let (b, d) = (stops.base.unit(), stops.deep.unit());
    let span = 2.0 * grid.side as f32;
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let along = (x as f32 + 0.5 - grid.origin as f32) + (y as f32 + 0.5 - grid.origin as f32);
        let t = (along / span).clamp(0.0, 1.0);
        Rgba([
            b[0] + (d[0] - b[0]) * t,
            b[1] + (d[1] - b[1]) * t,
            b[2] + (d[2] - b[2]) * t,
            1.0,
        ])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Family;

    /// 08 2.2's table, row by row.
    const GRIDS: &[(u32, u32, u32)] = &[
        (1024, 824, 100),
        (512, 412, 50),
        (64, 52, 6),
        (48, 39, 4),
        (32, 30, 1),
        (24, 22, 1),
        (16, 15, 0),
    ];

    #[test]
    fn grid_matches_the_table() {
        for (canvas, side, origin) in GRIDS {
            let g = PlateGrid::for_canvas(*canvas, &Template::default());
            assert_eq!((g.side, g.origin), (*side, *origin), "canvas {canvas}");
        }
    }

    /// 08 2.1: at n = 5 the squircle covers 95.0 % of its bounding square.
    #[test]
    fn squircle_area_is_95_percent() {
        let g = PlateGrid {
            canvas: 256,
            side: 256,
            origin: 0,
        };
        let area = plate_mask(g, &Template::default()).mean();
        assert!((area - 0.950).abs() < 0.003, "area {area}");
    }

    const POINTS: &[(&str, i64, i64, f32)] = &[
        ("centre", 512, 512, 1.0),
        ("canvas corner", 0, 0, 0.0),
        ("plate corner", 101, 101, 0.0),
        ("edge midpoint inside", 102, 512, 1.0),
    ];

    #[test]
    fn mask_points() {
        let m = plate_mask(
            PlateGrid::for_canvas(1024, &Template::default()),
            &Template::default(),
        );
        for (name, x, y, want) in POINTS {
            assert_eq!(m.at(*x, *y), *want, "{name}");
        }
    }

    #[test]
    fn gradient_runs_base_to_deep() {
        let g = PlateGrid::for_canvas(1024, &Template::default());
        let stops = Family::Blue.stops();
        let img = gradient(g, stops);
        let near = |p: [f32; 4], c: [f32; 3]| (0..3).all(|i| (p[i] - c[i]).abs() < 0.01);
        assert!(near(img.get_pixel(100, 100).0, stops.base.unit()));
        assert!(near(img.get_pixel(923, 923).0, stops.deep.unit()));
    }
}
