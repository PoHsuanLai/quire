//! Renders an abstract icon spec (design/08-ICONS.md 2.8) at any export size, natively: every
//! size is drawn from the shapes, not downscaled from the master, so 16 px edges stay crisp.

use image::{Rgba, Rgba32FImage};

use crate::{
    Layer, Lift, Plane, PlateGrid, Pt, STROKE, Spec, Srgb8, Template, apply_grain, compose::over,
    compose_on, gradient, mark::Outline, mark::distance, paint,
};

/// The cut-out's lift, Post `--shadow-1`'s drop (`0 1px 2px`, 07-LOOKS 3.2) read in the 24 grid
/// at the 48 px dock size: 0.5 unit down, 1 unit soft. The alpha is proposed.
const LIFT_OFFSET: f32 = 0.5;
const LIFT_SOFT: f32 = 1.0;
const LIFT_ALPHA: f32 = 0.16;
/// Grain is drawn from this export size up; below it one noise pixel is one icon pixel.
const GRAIN_FROM: u32 = 48;

/// Coverage of a signed distance `d` (grid units) at `k` px per unit, 1 px anti-aliasing.
fn cover(d: f32, k: f32) -> f32 {
    (0.5 - d * k).clamp(0.0, 1.0)
}

/// The distance whose inside is what the layer paints solid: the area, or the stroke band.
fn body(layer: &Layer, p: Pt) -> f32 {
    let d = distance(&layer.shape, p);
    match (layer.shape.outline(), layer.stroke) {
        (Outline::Line, _) => d - STROKE / 2.0,
        (Outline::Area, Some(_)) => d - STROKE / 2.0,
        (Outline::Area, None) => d,
    }
}

fn rgba(c: Srgb8, a: f32) -> [f32; 4] {
    let [r, g, b] = c.unit();
    [r, g, b, a]
}

/// One layer's colour at grid point `p` over `under`.
fn paint_layer(layer: &Layer, spec: &Spec, p: Pt, k: f32, under: [f32; 4]) -> [f32; 4] {
    let ink = Srgb8::hex(0x1A1E1A);
    let shadowed = match layer.lift {
        Lift::Lifted => {
            let d = body(layer, Pt(p.0, p.1 - LIFT_OFFSET));
            let a = (0.5 - d / (LIFT_SOFT + 1.0 / k)).clamp(0.0, 1.0) * LIFT_ALPHA;
            over(rgba(ink, a), under)
        }
        Lift::Flat => under,
    };
    let d = distance(&layer.shape, p);
    let filled = match (layer.shape.outline(), layer.fill) {
        (Outline::Area, Some(f)) => over(rgba(paint(f, spec.family), cover(d, k)), shadowed),
        _ => shadowed,
    };
    let band = match layer.shape.outline() {
        Outline::Line => d,
        Outline::Area => d.abs(),
    } - STROKE / 2.0;
    match layer.stroke {
        Some(s) => over(rgba(paint(s, spec.family), cover(band, k)), filled),
        None => filled,
    }
}

/// The object layers alone on transparent, at one canvas size.
pub fn emblem_object(spec: &Spec, grid: PlateGrid) -> Rgba32FImage {
    let k = grid.side as f32 / 24.0;
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let p = Pt(
            (x as f32 + 0.5 - grid.origin as f32) / k,
            (y as f32 + 0.5 - grid.origin as f32) / k,
        );
        Rgba(
            spec.layers
                .iter()
                .fold([0.0; 4], |under, l| paint_layer(l, spec, p, k, under)),
        )
    })
}

/// The shadowless icon (08 2.6 `flat`) at one canvas size: gradient plate, grain (from 48 px),
/// the layers, then the template's highlight and rim, clipped to the squircle.
pub fn emblem(spec: &Spec, canvas: u32, t: &Template, tile: &Plane) -> Rgba32FImage {
    let grid = PlateGrid::for_canvas(canvas, t);
    let plate = gradient(grid, spec.family.stops());
    let plate = match canvas >= GRAIN_FROM {
        true => apply_grain(&plate, spec.grain, tile),
        false => plate,
    };
    compose_on(grid, &plate, &emblem_object(spec, grid), t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grain_tile, parse_spec};

    const DOT: &str = r#"
name = "dot"
family = "blue"
grain = 0
[[layer]]
kind = "circle"
centre = [12, 12]
radius = 6
fill = "paper"
"#;

    #[test]
    fn a_paper_dot_on_a_blue_plate() {
        let spec = parse_spec(DOT).expect("spec");
        let t = Template::default();
        for size in [16, 48, 512] {
            let img = emblem(&spec, size, &t, &grain_tile());
            let c = img.get_pixel(size / 2, size / 2).0;
            assert!(c[0] > 0.95 && c[1] > 0.95, "{size}: centre is paper: {c:?}");
            let edge = img.get_pixel(size / 2, (size as f32 * 0.2) as u32).0;
            assert!(
                edge[2] > edge[0] + 0.3,
                "{size}: above the dot is the blue plate: {edge:?}"
            );
            assert_eq!(
                img.get_pixel(0, size - 1).0[3],
                0.0,
                "{size}: outside the squircle"
            );
        }
    }

    #[test]
    fn a_stroke_is_two_grid_units_wide() {
        let spec = parse_spec(
            r#"
name = "bar"
family = "green"
grain = 0
[[layer]]
kind = "bar"
from = [4, 12]
to = [20, 12]
stroke = "ink"
"#,
        )
        .expect("spec");
        let grid = PlateGrid {
            canvas: 240,
            side: 240,
            origin: 0,
        };
        let obj = emblem_object(&spec, grid);
        let column: f32 = (0..240).map(|y| obj.get_pixel(120, y).0[3]).sum();
        assert!(
            (column - 20.0).abs() < 0.6,
            "2 units at 10 px per unit: {column}"
        );
        assert!(
            obj.get_pixel(31, 120).0[3] > 0.99,
            "the cap reaches past the end"
        );
        assert!(
            obj.get_pixel(28, 120).0[3] < 0.01,
            "to x = 30 and no further"
        );
        assert!(
            obj.get_pixel(32, 111).0[3] < 0.5,
            "and is round: its corner is cut"
        );
    }
}
