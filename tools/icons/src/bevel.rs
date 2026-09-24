//! The round-three plate (design/08-ICONS.md 2.9): a two-hue matte face, and a finish that gives
//! the plate thickness: a narrow highlight arc along the top of the bevel, a soft shade along the
//! bottom, one small soft specular point near the top-left corner, clipped to the squircle.

use image::{Rgba, Rgba32FImage};

use crate::{Ground, Plane, PlateGrid, Template, compose::over, oklab, plate::superellipse, srgb};

/// The face's matte diffusion: the top of the plate is this much lighter than the middle and the
/// bottom this much darker (a gentle light from above, never a hot spot).
const DIFFUSION: f32 = 0.05;
/// The ground is soft: its hues keep this share of their Candy chroma and are lifted this much
/// in lightness, so a two-hue face reads as matte colour, not as a saturated sticker.
const GROUND_CHROMA: f32 = 0.78;
const GROUND_LIFT: f32 = 0.03;
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

/// OKLCh mix of two colours along the shorter hue arc, so two distant hues meet through a
/// saturated middle (violet to amber passes through rose, not through grey).
fn mix_lch(p: crate::Oklab, q: crate::Oklab, t: f32) -> crate::Oklab {
    let (cp, cq) = (p.a.hypot(p.b), q.a.hypot(q.b));
    let (hp, hq) = (p.b.atan2(p.a), q.b.atan2(q.a));
    let tau = std::f32::consts::TAU;
    let dh = (hq - hp + tau * 1.5).rem_euclid(tau) - tau / 2.0;
    let (c, h) = (cp + (cq - cp) * t, hp + dh * t);
    crate::Oklab {
        l: p.l + (q.l - p.l) * t,
        a: c * h.cos(),
        b: c * h.sin(),
    }
}

/// The two-hue ground over the whole canvas, 135deg across the plate, mixed in OKLCh, with the
/// matte diffusion applied. Opaque; [`finish`] masks it.
pub fn ground_face(grid: PlateGrid, ground: Ground) -> Rgba32FImage {
    let (a, b) = (oklab(ground.start.0.unit()), oklab(ground.end.0.unit()));
    let side = grid.side as f32;
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let (u, v) = (
            x as f32 + 0.5 - grid.origin as f32,
            y as f32 + 0.5 - grid.origin as f32,
        );
        let t = ((u + v) / (2.0 * side)).clamp(0.0, 1.0);
        let mut c = mix_lch(a, b, t);
        c.l += GROUND_LIFT + DIFFUSION * (1.0 - 2.0 * (v / side).clamp(0.0, 1.0));
        c.a *= GROUND_CHROMA;
        c.b *= GROUND_CHROMA;
        let [r, g, bl] = srgb(c);
        Rgba([r, g, bl, 1.0])
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
    use crate::{Colour, Srgb8};

    fn ground() -> Ground {
        Ground {
            start: Colour(Srgb8::hex(0x8B5CF0)),
            end: Colour(Srgb8::hex(0xF0A81E)),
        }
    }

    #[test]
    fn face_runs_start_to_end_and_is_lighter_on_top() {
        let g = PlateGrid::for_canvas(256, &Template::default());
        let face = ground_face(g, ground());
        let tl = face.get_pixel(g.origin + 2, g.origin + 2).0;
        let br = face
            .get_pixel(g.origin + g.side - 3, g.origin + g.side - 3)
            .0;
        assert!(tl[2] > tl[1], "violet at the top-left: {tl:?}");
        assert!(
            br[0] > br[2] && br[1] > br[2],
            "amber at the bottom-right: {br:?}"
        );
        let mid = |x: u32, y: u32| oklab([0, 1, 2].map(|i| face.get_pixel(x, y).0[i])).l;
        // Along one anti-diagonal the hue mix is constant, so only the diffusion differs.
        let (x0, y0) = (g.origin + g.side * 3 / 4, g.origin + g.side / 4);
        let (x1, y1) = (g.origin + g.side / 4, g.origin + g.side * 3 / 4);
        assert!(
            mid(x0, y0) > mid(x1, y1),
            "top lighter than bottom on the same mix"
        );
    }

    #[test]
    fn finish_lights_the_top_edge_and_shades_the_bottom() {
        let t = Template::default();
        let g = PlateGrid::for_canvas(512, &t);
        let face = ground_face(g, ground());
        let flat = finish(g, &face, &t, Bevel::Ours);
        let at = |x: u32, y: u32| oklab([0, 1, 2].map(|i| flat.get_pixel(x, y).0[i])).l;
        let face_l = |x: u32, y: u32| oklab([0, 1, 2].map(|i| face.get_pixel(x, y).0[i])).l;
        let cx = g.origin + g.side / 2;
        assert!(
            at(cx, g.origin + 2) > face_l(cx, g.origin + 2) + 0.02,
            "arc on top"
        );
        let bottom = g.origin + g.side - 3;
        assert!(
            at(cx, bottom) < face_l(cx, bottom) - 0.01,
            "shade at the bottom"
        );
        assert_eq!(flat.get_pixel(0, 0).0[3], 0.0, "outside the squircle");
        let theirs = finish(g, &face, &t, Bevel::Theirs);
        let l = oklab([0, 1, 2].map(|i| theirs.get_pixel(cx, g.origin + 2).0[i])).l;
        assert!(
            l <= face_l(cx, g.origin + 2),
            "no arc when the tile brings its own"
        );
    }
}
