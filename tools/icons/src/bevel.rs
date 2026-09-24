//! The round-three plate (design/08-ICONS.md 2.9): a two-hue matte face, and a finish that gives
//! the plate thickness: a narrow highlight arc along the top of the bevel, a soft shade along the
//! bottom, one small soft specular point near the top-left corner, clipped to the squircle.

use image::{Rgba, Rgba32FImage};

use crate::{Oklab, Plane, PlateGrid, Shift, Template, compose::over, plate::superellipse, srgb};

/// The face's matte diffusion: the top of the plate is this much lighter (OKLab L) than the
/// middle and the bottom this much darker (a gentle light from above, never a hot spot).
const DIFFUSION: f32 = 0.03;
/// The only gradient left (08 2.10): an almost invisible tonal shift within the plate's one hue,
/// this much lighter at the top-left corner and as much darker at the bottom-right.
const TONAL_SHIFT: f32 = 0.012;
/// Bevel highlight and shade, and the specular point (alphas, proposed).
const ARC_ALPHA: f32 = 0.55;
const SHADE_ALPHA: f32 = 0.14;
const POINT_ALPHA: f32 = 0.28;
const RIM_ALPHA: f32 = 0.06;
/// The specular point and the soft bevel are drawn from this export size up.
const DETAIL_FROM: u32 = 48;

/// Whether the finish draws our bevel or only masks (a model-drawn tile brings its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bevel {
    Ours,
    Theirs,
}

/// The plate's face over the whole canvas: one colour, the tonal shift across the plate at
/// 135deg and the matte diffusion from above. Opaque; [`finish`] masks it.
pub fn plate_face(grid: PlateGrid, colour: Oklab, shift: Shift) -> Rgba32FImage {
    let side = grid.side as f32;
    let tonal = match shift {
        Shift::Tonal => TONAL_SHIFT,
        Shift::Flat => 0.0,
    };
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let (u, v) = (
            x as f32 + 0.5 - grid.origin as f32,
            y as f32 + 0.5 - grid.origin as f32,
        );
        let t = ((u + v) / (2.0 * side)).clamp(0.0, 1.0);
        let lift = tonal * (1.0 - 2.0 * t) + DIFFUSION * (1.0 - 2.0 * (v / side).clamp(0.0, 1.0));
        let [r, g, b] = srgb(Oklab {
            l: colour.l + lift,
            ..colour
        });
        Rgba([r, g, b, 1.0])
    })
}

fn tint(coverage: &Plane, colour: [f32; 3], alpha: f32) -> impl Fn(u32, u32) -> [f32; 4] + '_ {
    move |x, y| {
        [
            colour[0],
            colour[1],
            colour[2],
            coverage.at(i64::from(x), i64::from(y)) * alpha,
        ]
    }
}

/// The finished `flat` icon: `face` (ground plus symbol) with the bevel and rim, clipped to the
/// squircle.
pub fn finish(grid: PlateGrid, face: &Rgba32FImage, t: &Template, bevel: Bevel) -> Rgba32FImage {
    let (c, a) = (grid.centre(), grid.half());
    let mask = superellipse(grid.canvas, (c, c), a, t.exponent);
    let s = (grid.side as f32 / 48.0).max(1.0);
    let w = (0.7 * s).round().max(1.0) as i64;
    let detail = grid.canvas >= DETAIL_FROM;
    let edge = |dy: i64| {
        let band = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
            let (x, y) = (i64::from(x), i64::from(y));
            mask.at(x, y) * (1.0 - mask.at(x, y + dy))
        });
        match detail {
            true => band.blurred(0.5 * w as f32),
            false => band,
        }
    };
    // The arc: the top band, strongest in the middle, fading towards both sides.
    let top = edge(-w);
    let arc = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
        let f = ((x as f32 + 0.5 - c) / (0.95 * a)).powi(2);
        top.at(i64::from(x), i64::from(y)) * (1.0 - f).max(0.0)
    });
    let shade = edge(w);
    let inner = superellipse(grid.canvas, (c, c), a - s.min(2.0), t.exponent);
    let rim = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
        let (x, y) = (i64::from(x), i64::from(y));
        mask.at(x, y) * (1.0 - inner.at(x, y))
    });
    // The specular point: on the bevel where the squircle's diagonal meets it, top-left.
    let k = 0.5_f32.powf(1.0 / t.exponent);
    let (px, py) = (c - (a * k - 2.0 * s), c - (a * k - 2.0 * s));
    let r = 1.1 * s;
    let point = Plane::from_fn(grid.canvas, grid.canvas, |x, y| {
        let d2 = (x as f32 + 0.5 - px).powi(2) + (y as f32 + 0.5 - py).powi(2);
        match detail {
            true => (-d2 / (2.0 * r * r)).exp(),
            false => 0.0,
        }
    });
    let white = [1.0; 3];
    let dark = [0.10, 0.12, 0.10];
    let arc_px = tint(&arc, white, ARC_ALPHA);
    let shade_px = tint(&shade, dark, SHADE_ALPHA);
    let point_px = tint(&point, white, POINT_ALPHA);
    let rim_px = tint(&rim, dark, RIM_ALPHA);
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let base = face.get_pixel(x, y).0;
        let p = match bevel {
            Bevel::Ours => [shade_px(x, y), rim_px(x, y), arc_px(x, y), point_px(x, y)]
                .into_iter()
                .fold(base, |under, top| over(top, under)),
            Bevel::Theirs => over(rim_px(x, y), base),
        };
        Rgba([p[0], p[1], p[2], p[3] * mask.at(i64::from(x), i64::from(y))])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oklab;

    const SLATE: Oklab = Oklab {
        l: 0.62,
        a: -0.015,
        b: -0.05,
    };

    fn l_at(img: &Rgba32FImage, x: u32, y: u32) -> f32 {
        oklab([0, 1, 2].map(|i| img.get_pixel(x, y).0[i])).l
    }

    #[test]
    fn face_is_one_colour_with_an_almost_invisible_shift() {
        let g = PlateGrid::for_canvas(256, &Template::default());
        let face = plate_face(g, SLATE, Shift::Tonal);
        let (tl, br) = (
            l_at(&face, g.origin + 2, g.origin + 2),
            l_at(&face, g.origin + g.side - 3, g.origin + g.side - 3),
        );
        assert!(tl > br, "lighter top-left");
        assert!(
            tl - br < 2.0 * (TONAL_SHIFT + DIFFUSION) + 0.01,
            "but only a shift: {tl} {br}"
        );
        let mid = face.get_pixel(128, 128).0;
        let hue = |p: [f32; 4]| {
            let o = oklab([p[0], p[1], p[2]]);
            o.b.atan2(o.a)
        };
        assert!((hue(mid) - hue(face.get_pixel(g.origin + 2, g.origin + 2).0)).abs() < 0.05);
    }

    #[test]
    fn finish_lights_the_top_edge_and_shades_the_bottom() {
        let t = Template::default();
        let g = PlateGrid::for_canvas(512, &t);
        let face = plate_face(g, SLATE, Shift::Tonal);
        let flat = finish(g, &face, &t, Bevel::Ours);
        let cx = g.origin + g.side / 2;
        assert!(
            l_at(&flat, cx, g.origin + 2) > l_at(&face, cx, g.origin + 2) + 0.02,
            "arc"
        );
        let bottom = g.origin + g.side - 3;
        assert!(
            l_at(&flat, cx, bottom) < l_at(&face, cx, bottom) - 0.01,
            "shade"
        );
        assert_eq!(flat.get_pixel(0, 0).0[3], 0.0, "outside the squircle");
        let theirs = finish(g, &face, &t, Bevel::Theirs);
        assert!(
            l_at(&theirs, cx, g.origin + 2) <= l_at(&face, cx, g.origin + 2),
            "no arc"
        );
    }
}
