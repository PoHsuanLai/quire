//! `--level-sheet DIR`: the level control's contact sheets for the user's pick (the brief of
//! 2026-09-25). `level-variants.png`: each look (capsule, capsule and knob, segments) in light
//! and dark, over the Work Space's tint and over a light ground, for volume at 0, 40 and 100 %,
//! muted, and brightness at 30 %, at 1x and at 2x. `level-motion.png`: headless frames through a
//! level set from outside, a press, a drag past the end and the release (`level_motion.rs`).
//! Both are written to DIR and copied to the progress page's shots.

use crate::error::GalleryError;
use crate::pages::level_tile::{Ground, LevelTile, STATES};
use crate::snapshot::progress_dir;
use crate::style;
use dioxus::prelude::*;
use ds::{Appearance, Ds, LevelLook, Material, Scheme, Theme};
use ds_native::{Harness, Viewport};
use image::{Rgba, RgbaImage, imageops};
use std::cell::Cell;
use std::path::Path;
use std::time::Duration;

/// A row's width in logical pixels: a label column and five tiles.
const WIDTH: u32 = 2020;
/// Tall enough for a row; the picture is cropped to it.
const HEIGHT: u32 = 160;
/// Between the 1x and the 2x sheet, and between looks.
const GAP: u32 = 24;

thread_local! {
    /// The row [`Row`] draws: one look, scheme and ground.
    static ROW: Cell<(LevelLook, Scheme, Ground)> =
        const { Cell::new((LevelLook::Capsule, Scheme::Light, Ground::Work)) };
}

/// One row of the sheet: its label, then a tile per state. A row is rendered on its own: Blitz's
/// CPU renderer loses layers when one document holds every tile (FINDINGS "Level control").
#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Row() -> Element {
    let (look, scheme, ground) = ROW.get();
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window,
            style { {style::CSS} }
            div { class: "g-level-sheet",
                div { class: "g-row g-row-top",
                    div { class: "g-col g-level-label",
                        span { class: "g-name", "{look.slug()}" }
                        span { class: "g-code", "{scheme.slug()}, {ground.label()}" }
                    }
                    for state in STATES {
                        LevelTile { look, scheme, ground, state }
                    }
                }
            }
        }
    }
}

/// One row at `scale_percent`, cropped to its content.
fn row_at(row: (LevelLook, Scheme, Ground), scale_percent: u16) -> Result<RgbaImage, GalleryError> {
    ROW.set(row);
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
        scale_percent,
    };
    let mut harness = Harness::new(Row, viewport);
    harness.advance(Duration::from_millis(600));
    let sheet = harness.rect(".g-level-sheet");
    let picture = harness.render().map_err(|source| GalleryError::Render {
        name: "level-variants".into(),
        source,
    })?;
    let scale = f32::from(scale_percent) / 100.0;
    let height = sheet.map_or(HEIGHT as f32, |rect| rect.origin.y.0 + rect.size.height.0);
    let tall = ((height * scale).ceil() as u32).min(picture.height());
    Ok(imageops::crop_imm(&picture, 0, 0, picture.width(), tall).to_image())
}

/// Every row at `scale_percent`, look by look.
fn variants_at(scale_percent: u16) -> Result<RgbaImage, GalleryError> {
    let looks = LevelLook::ALL
        .into_iter()
        .map(|look| {
            let rows = [Scheme::Light, Scheme::Dark]
                .into_iter()
                .flat_map(|scheme| Ground::ALL.map(|ground| (look, scheme, ground)))
                .map(|row| row_at(row, scale_percent))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(stacked(&rows, 0))
        })
        .collect::<Result<Vec<_>, GalleryError>>()?;
    Ok(stacked(&looks, GAP))
}

/// `pictures` one under another on a white ground, `gap` apart.
pub(crate) fn stacked(pictures: &[RgbaImage], gap: u32) -> RgbaImage {
    let width = pictures.iter().map(RgbaImage::width).max().unwrap_or(1);
    let height = pictures.iter().map(RgbaImage::height).sum::<u32>()
        + gap * pictures.len().saturating_sub(1) as u32;
    let mut sheet = RgbaImage::from_pixel(width, height.max(1), Rgba([255, 255, 255, 255]));
    let mut y = 0i64;
    for picture in pictures {
        imageops::overlay(&mut sheet, picture, 0, y);
        y += i64::from(picture.height() + gap);
    }
    sheet
}

/// Write `picture` as `name` in `dir` and copy it to the progress page's shots.
pub(crate) fn keep(picture: &RgbaImage, dir: &Path, name: &str) -> Result<(), GalleryError> {
    let made = |path: &Path| {
        let path = path.to_path_buf();
        move |source| GalleryError::Write { path, source }
    };
    std::fs::create_dir_all(dir).map_err(made(dir))?;
    let path = dir.join(name);
    picture.save(&path).map_err(|source| GalleryError::Encode {
        path: path.clone(),
        source,
    })?;
    let progress = progress_dir();
    std::fs::create_dir_all(&progress).map_err(made(&progress))?;
    let copy = progress.join(name);
    std::fs::copy(&path, &copy).map_err(made(&copy))?;
    eprintln!("{} (and {})", path.display(), copy.display());
    Ok(())
}

/// Render both sheets into `dir`.
pub fn run(dir: &Path) -> Result<(), GalleryError> {
    let variants = stacked(&[variants_at(100)?, variants_at(200)?], GAP);
    keep(&variants, dir, "level-variants.png")?;
    let motion = crate::level_motion::strip()?;
    keep(&motion, dir, "level-motion.png")
}
