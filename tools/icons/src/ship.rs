//! The shipped set (design/08-ICONS.md 2.11): a manifest naming each app's spec, dialect, hue
//! and source, and the renderers that draw every export size of every icon style directly.
//!
//! A procedural icon is drawn from its shapes at each size. A Klein-derived one starts from a
//! round-three render (never committed): it is retinted once at full size into the dialect's
//! roles, and at each size the face is resampled onto the plate while the plate, grain, bevel,
//! rim and shadow are drawn at that size by the same code as the procedural icons.

use image::imageops::{FilterType, crop_imm, resize};
use image::{Rgba, Rgba32FImage};
use serde::Deserialize;

use crate::{
    Bevel, ChromaCap, Dialect, Look, Plane, PlateGrid, Spec, Template, Tint, apply_grain,
    drop_shadow, finish, grid_for, render_icon, retint, roles,
};

/// Every pixel size the set exports, each drawn directly: the freedesktop sizes 16, 22, 24, 32,
/// 48, 64, 128, 256, 512; their `@2` up to 256@2 as 08 2.6 settles (32, 44, 48, 64, 96, 128,
/// 256, 512); and the 1.5x sizes 24 -> 36, 32 -> 48, 48 -> 72, 64 -> 96. A size shared by
/// several (48 is 48, 24@2 and 32 at 1.5x) is one file. 512@2 (1024) is not exported: 08 2.6
/// stops `@2` at 256, and the grain makes a 1024 PNG about 1.2 MB.
pub const SHIP_PX: [u32; 13] = [16, 22, 24, 32, 36, 44, 48, 64, 72, 96, 128, 256, 512];

/// Where an icon's face comes from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Source {
    /// Drawn from the spec's shapes.
    Procedural,
    /// A round-three Klein face-mode render (a file name under the renders directory),
    /// retinted into the dialect.
    KleinRetint { render: String },
}

/// One shipped app.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ShipApp {
    pub name: String,
    /// The spec, relative to the manifest.
    pub spec: String,
    pub dialect: Dialect,
    pub hue: Tint,
    pub source: Source,
}

/// `tools/icons/ship.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    /// The shipped set's chroma cap (the palette's, 0.07).
    pub chroma: f32,
    /// The Muted style's cap (0.04).
    pub muted_chroma: f32,
    #[serde(rename = "app")]
    pub apps: Vec<ShipApp>,
}

/// The three exported styles (`icons.style`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Colour,
    Muted,
    Monochrome,
}

impl Style {
    pub const ALL: [Style; 3] = [Style::Colour, Style::Muted, Style::Monochrome];

    /// The directory under `assets/icons/apps/<app>/`; the Colour set sits in the app's own.
    pub const fn dir(self) -> Option<&'static str> {
        match self {
            Style::Colour => None,
            Style::Muted => Some("muted"),
            Style::Monochrome => Some("monochrome"),
        }
    }
}

/// The look an app takes in a style. Monochrome is exported neutral (no hue): it is tinted at
/// run time by `ds::icon::retint` from `icons.monochrome_tint`.
pub fn style_look(app: &ShipApp, style: Style, m: &Manifest) -> Look {
    match style {
        Style::Colour => Look {
            dialect: app.dialect,
            tint: app.hue,
            cap: ChromaCap(m.chroma),
        },
        Style::Muted => Look {
            dialect: app.dialect,
            tint: app.hue,
            cap: ChromaCap(m.muted_chroma),
        },
        Style::Monochrome => Look {
            dialect: Dialect::Monochrome,
            tint: Tint {
                hue: app.hue.hue,
                strength: 0.0,
            },
            cap: ChromaCap(m.chroma),
        },
    }
}

/// The share of a full-bleed Klein face render kept as the plate face (round three).
pub const FACE_CROP: f32 = 0.78;

/// A Klein face retinted into a look, at its render's size.
pub fn klein_face(raw: &Rgba32FImage, look: Look) -> Rgba32FImage {
    let side = (raw.width().min(raw.height()) as f32 * FACE_CROP) as u32;
    let (x0, y0) = ((raw.width() - side) / 2, (raw.height() - side) / 2);
    let crop = crop_imm(raw, x0, y0, side, side).to_image();
    // A Klein symbol is always light on its ground, so the Monochrome roles apply whatever the
    // app's dialect (the manifest only pairs Klein faces with Monochrome).
    retint(&crop, &roles(Dialect::Monochrome, look.tint, look.cap))
}

/// One size of a Klein-derived icon: the face resampled onto the size's plate, the grain from
/// 48 px, then the same finish and shadow as a procedural icon.
pub fn face_icon(face: &Rgba32FImage, size: u32, t: &Template, tile: &Plane) -> Rgba32FImage {
    let grid = grid_for(size, t);
    let small = resize(face, grid.side, grid.side, FilterType::Lanczos3);
    let placed = placed(&small, grid);
    let placed = match size >= 48 {
        true => apply_grain(&placed, crate::Grain(20), tile),
        false => placed,
    };
    let flat = finish(grid, &placed, t, Bevel::Ours);
    match size >= 48 {
        true => drop_shadow(&flat, grid, t),
        false => flat,
    }
}

fn placed(small: &Rgba32FImage, grid: PlateGrid) -> Rgba32FImage {
    let inside = grid.origin..grid.origin + grid.side;
    Rgba32FImage::from_fn(grid.canvas, grid.canvas, |x, y| {
        match inside.contains(&x) && inside.contains(&y) {
            true => *small.get_pixel(x - grid.origin, y - grid.origin),
            false => Rgba([0.0; 4]),
        }
    })
}

/// What an app's icon is drawn from in one style.
#[derive(Debug, Clone)]
pub enum Prepared<'a> {
    Procedural(&'a Spec, Look),
    Face(Rgba32FImage),
}

/// One export size of a prepared icon.
pub fn ship_icon(p: &Prepared, size: u32, t: &Template, tile: &Plane) -> Rgba32FImage {
    match p {
        Prepared::Procedural(spec, look) => render_icon(spec, *look, size, t, tile),
        Prepared::Face(face) => face_icon(face, size, t, tile),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = include_str!("../ship.toml");

    #[test]
    fn manifest_names_the_five_apps() {
        let m: Manifest = toml::from_str(MANIFEST).expect("ship.toml");
        let names: Vec<_> = m.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["mail", "files", "terminal", "notes", "photos"]);
        assert!((m.chroma - 0.07).abs() < 1e-6 && (m.muted_chroma - 0.04).abs() < 1e-6);
        let klein: Vec<_> = m
            .apps
            .iter()
            .filter(|a| matches!(a.source, Source::KleinRetint { .. }))
            .map(|a| (a.name.as_str(), a.dialect))
            .collect();
        assert_eq!(
            klein,
            [
                ("mail", Dialect::Monochrome),
                ("photos", Dialect::Monochrome)
            ],
            "Klein faces are only paired with Monochrome"
        );
    }

    #[test]
    fn sizes_are_unique_and_cover_the_scales() {
        for n in [16, 22, 24, 32, 48, 64, 128, 256, 512] {
            assert!(SHIP_PX.contains(&n), "{n}");
        }
        for n in [16, 22, 24, 32, 48, 64, 128, 256] {
            assert!(SHIP_PX.contains(&(2 * n)), "{n}@2");
        }
        for n in [24, 32, 48, 64] {
            assert!(SHIP_PX.contains(&(n * 3 / 2)), "{n} at 1.5x");
        }
        let mut sorted = SHIP_PX.to_vec();
        sorted.dedup();
        assert_eq!(sorted.len(), SHIP_PX.len());
    }

    #[test]
    fn a_face_icon_has_the_plate_silhouette() {
        let t = Template::default();
        let face = Rgba32FImage::from_pixel(64, 64, Rgba([0.5, 0.5, 0.6, 1.0]));
        for size in [16, 48, 128] {
            let img = face_icon(&face, size, &t, &crate::grain_tile());
            let mask = crate::plate_mask(grid_for(size, &t), &t);
            let c = size / 2;
            assert!(img.get_pixel(c, c).0[3] > 0.99 && mask.at(0, 0) == 0.0);
            assert!(img.get_pixel(0, 0).0[3] < 0.05, "{size}: corner outside");
        }
    }
}
