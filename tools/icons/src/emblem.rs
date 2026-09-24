//! Renders an abstract icon spec (design/08-ICONS.md 2.8-2.10) at any export size, natively:
//! every size is drawn from the shapes, not downscaled from the master, so 16 px edges stay
//! crisp. Layers are filled and pressed into the plate: a raised layer has a light inner edge on
//! top, a soft darker inner edge below and a faint seat under it; a recessed one the reverse.
//! Below [`EMBOSS_FROM`] the emboss is dropped and only the flat silhouettes remain.

use image::{Rgba, Rgba32FImage};

use crate::{
    Bevel, ChromaCap, Dialect, Layer, Oklab, Plane, PlateGrid, Pt, Relief, Role, Roles, Spec,
    Srgb8, Template, Tint, apply_grain, compose::over, finish, mark::distance, plate_face, roles,
    srgb,
};

/// What an icon is drawn as: a dialect under a tint and a chroma cap (the spec's own dialect
/// and tint, or overrides).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub dialect: Dialect,
    pub tint: Tint,
    pub cap: ChromaCap,
}

impl Look {
    /// The look the spec recommends.
    pub fn of(spec: &Spec) -> Look {
        Look {
            dialect: spec.dialect,
            tint: spec.tint,
            cap: ChromaCap::default(),
        }
    }
}

/// The emboss is drawn from this export size up; at 16 and 32 it is noise, not form.
pub const EMBOSS_FROM: u32 = 48;
/// Grain is drawn from this export size up; below it one noise pixel is one icon pixel.
const GRAIN_FROM: u32 = 48;
/// Emboss geometry in grid units: how far the light and shade reach in from the edge, and how
/// soft they are. Alphas are proposed.
const DEPTH: f32 = 0.45;
const SOFT: f32 = 0.8;
const LIGHT_ALPHA: f32 = 0.55;
const SHADE_ALPHA: f32 = 0.20;
const SEAT_ALPHA: f32 = 0.14;

/// Coverage of a signed distance `d` (grid units) at `k` px per unit, 1 px anti-aliasing.
fn cover(d: f32, k: f32) -> f32 {
    (0.5 - d * k).clamp(0.0, 1.0)
}

/// How far outside the shape a point is, softened: 0 deep inside, 1 well outside.
fn outside(d: f32, soft: f32) -> f32 {
    (0.5 + d / soft).clamp(0.0, 1.0)
}

fn rgba(c: Srgb8, a: f32) -> [f32; 4] {
    let [r, g, b] = c.unit();
    [r, g, b, a]
}

fn lab_rgba(c: Oklab, a: f32) -> [f32; 4] {
    let [r, g, b] = srgb(c);
    [r, g, b, a]
}

fn role(r: &Roles, which: Role) -> Oklab {
    match which {
        Role::Symbol => r.symbol,
        Role::Secondary => r.secondary,
        Role::Detail => r.detail,
        Role::Spot => r.spot,
    }
}

/// Whether the emboss is drawn at this size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emboss {
    On,
    Off,
}

/// One layer's colour at grid point `p` over `under`.
fn paint_layer(
    layer: &Layer,
    r: &Roles,
    p: Pt,
    k: f32,
    emboss: Emboss,
    under: [f32; 4],
) -> [f32; 4] {
    let ink = Srgb8::hex(0x1A1E1A);
    let white = Srgb8::hex(0xFFFFFF);
    let d = distance(&layer.shape, p);
    let inside = cover(d, k);
    let fill = lab_rgba(role(r, layer.fill), inside);
    let (relief, soft) = match emboss {
        Emboss::On => (layer.relief, SOFT + 1.0 / k),
        Emboss::Off => (Relief::Flush, SOFT),
    };
    let above = outside(distance(&layer.shape, Pt(p.0, p.1 - DEPTH)), soft);
    let below = outside(distance(&layer.shape, Pt(p.0, p.1 + DEPTH)), soft);
    let (top, bottom, seat) = match relief {
        Relief::Raised => {
            let seat = (1.0 - inside)
                * (1.0 - outside(distance(&layer.shape, Pt(p.0, p.1 - DEPTH)), soft));
            (
                rgba(white, inside * above * LIGHT_ALPHA),
                rgba(ink, inside * below * SHADE_ALPHA),
                rgba(ink, seat * SEAT_ALPHA),
            )
        }
        Relief::Recessed => (
            rgba(ink, inside * above * SHADE_ALPHA * 1.2),
            rgba(white, inside * below * LIGHT_ALPHA * 0.8),
            [0.0; 4],
        ),
        Relief::Flush => ([0.0; 4], [0.0; 4], [0.0; 4]),
    };
    [seat, fill, bottom, top]
        .into_iter()
        .fold(under, |u, t| over(t, u))
}

/// The layers alone on transparent, at one canvas size, in the given roles' colours.
pub fn emblem_object(spec: &Spec, r: &Roles, grid: PlateGrid) -> Rgba32FImage {
    let k = grid.side as f32 / 24.0;
    let emboss = match grid.canvas >= EMBOSS_FROM {
        true => Emboss::On,
        false => Emboss::Off,
    };
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        let p = Pt(
            (x as f32 + 0.5 - grid.origin as f32) / k,
            (y as f32 + 0.5 - grid.origin as f32) / k,
        );
        Rgba(
            spec.layers
                .iter()
                .fold([0.0; 4], |under, l| paint_layer(l, r, p, k, emboss, under)),
        )
    })
}

/// The shadowless icon (08 2.6 `flat`) at one canvas size in one look: the dialect's plate with
/// its tonal shift, grain (from 48 px), the pressed layers, then the bevel finish, clipped to
/// the squircle.
pub fn emblem(spec: &Spec, look: Look, canvas: u32, t: &Template, tile: &Plane) -> Rgba32FImage {
    let grid = PlateGrid::for_canvas(canvas, t);
    let r = roles(look.dialect, look.tint, look.cap);
    let face = plate_face(grid, r.plate, r.shift);
    let face = match canvas >= GRAIN_FROM {
        true => apply_grain(&face, spec.grain, tile),
        false => face,
    };
    let object = emblem_object(spec, &r, grid);
    let face = Rgba32FImage::from_fn(canvas, canvas, |x, y| {
        Rgba(over(object.get_pixel(x, y).0, face.get_pixel(x, y).0))
    });
    finish(grid, &face, t, Bevel::Ours)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grain_tile, oklab, parse_spec};

    const DOT: &str = r#"
name = "dot"
dialect = "solid"
tint = "slate"
grain = 0
[[layer]]
kind = "circle"
centre = [12, 12]
radius = 6
fill = "symbol"
"#;

    fn l(p: [f32; 4]) -> f32 {
        oklab([p[0], p[1], p[2]]).l
    }

    /// Every dialect draws the dot apart from its plate by lightness, at every size.
    #[test]
    fn a_dot_in_every_dialect() {
        let spec = parse_spec(DOT).expect("spec");
        let t = Template::default();
        for dialect in Dialect::ALL {
            let look = Look {
                dialect,
                tint: spec.tint,
                cap: ChromaCap::default(),
            };
            for size in [16, 48, 512] {
                let img = emblem(&spec, look, size, &t, &grain_tile());
                let c = img.get_pixel(size / 2, size / 2).0;
                // Left of the dot, clear of the top bevel arc.
                let edge = img.get_pixel((size as f32 * 0.18) as u32, size / 2).0;
                assert!(
                    (l(c) - l(edge)).abs() > 0.2,
                    "{dialect:?} {size}: {c:?} {edge:?}"
                );
                assert_eq!(
                    img.get_pixel(0, size - 1).0[3],
                    0.0,
                    "{size}: outside the squircle"
                );
            }
        }
    }

    /// Raised: lighter just inside the top edge than just inside the bottom edge; no outline.
    #[test]
    fn raised_layers_are_lit_on_top_and_shaded_below() {
        let spec = parse_spec(DOT).expect("spec");
        let r = roles(Dialect::Monochrome, spec.tint, ChromaCap::default());
        let grid = PlateGrid {
            canvas: 240,
            side: 240,
            origin: 0,
        };
        let obj = emblem_object(&spec, &r, grid);
        let at = |x: u32, y: u32| l(obj.get_pixel(x, y).0);
        // Radius 6 units = 60 px round (120, 120): top edge at y = 60, bottom at 180.
        assert!(at(120, 64) > at(120, 120) + 0.02, "light inner edge on top");
        assert!(at(120, 176) < at(120, 120) - 0.03, "shade inner edge below");
        assert!(
            obj.get_pixel(120, 186).0[3] < 0.2,
            "the seat below is faint"
        );
        assert!(
            obj.get_pixel(120, 50).0[3] < 0.01,
            "nothing drawn above the shape"
        );
    }

    #[test]
    fn small_sizes_drop_the_emboss() {
        let spec = parse_spec(DOT).expect("spec");
        let r = roles(Dialect::Monochrome, spec.tint, ChromaCap::default());
        let obj = emblem_object(&spec, &r, PlateGrid::for_canvas(32, &Template::default()));
        let (a, b) = (obj.get_pixel(16, 11).0, obj.get_pixel(16, 20).0);
        assert!(
            (a[0] - b[0]).abs() < 1e-6,
            "flat fill top and bottom: {a:?} {b:?}"
        );
    }
}
