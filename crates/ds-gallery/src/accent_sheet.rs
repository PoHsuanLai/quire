//! `--accent-sheet DIR`: the accent candidates for the user's pick (design/03-COLOR.md section
//! 20, proposed). One column per accent: Postmark as it ships (the baseline), then each
//! candidate band at Postmark's hue. Every column draws the same surfaces, light and dark, over
//! the calm wallpaper and a material: the primary button, a toggle that is on, the segmented
//! control, a chip, a link, the focus ring, a menu's highlight, a selected row, the control
//! center's tiles, the compact calendar (month title, today disc) and the launcher's selected
//! row; under them the six built-in hues. Writes `accent-candidates-{light,dark}.png` and one
//! crop per column and scheme, in DIR and in the progress page's `shots/accent`.

use crate::accent_specimens::{Column, Specimens};
use crate::error::GalleryError;
use crate::style;
use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Scheme, Theme};
use ds_native::{Harness, Viewport};
use image::{Rgba, RgbaImage, imageops};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A column's width in logical pixels.
const WIDTH: u32 = 380;
/// Tall enough for a column; the picture is cropped to it.
const HEIGHT: u32 = 1500;
/// Between columns.
const GAP: u32 = 16;
/// The device scale: 2x, so the rings and hairlines read.
const SCALE: u16 = 200;

thread_local! {
    /// The column [`Sheet`] draws.
    static COLUMN: Cell<(Column, Scheme)> = const { Cell::new((Column::Postmark, Scheme::Light)) };
}

/// One column, rendered on its own (Blitz's CPU renderer loses layers in one large document).
#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Sheet() -> Element {
    let (column, scheme) = COLUMN.get();
    let theme = match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    };
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window,
            style { {style::CSS} }
            Specimens { column, scheme }
        }
    }
}

/// One column in `scheme`, cropped to its content.
fn column_at(column: Column, scheme: Scheme) -> Result<RgbaImage, GalleryError> {
    COLUMN.set((column, scheme));
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
        scale_percent: SCALE,
    };
    let mut harness = Harness::new(Sheet, viewport);
    harness.advance(Duration::from_secs(2));
    let sheet = harness.rect(".g-acc");
    let picture = harness.render().map_err(|source| GalleryError::Render {
        name: format!("accent {}", column.slug()),
        source,
    })?;
    let scale = f32::from(SCALE) / 100.0;
    let height = sheet.map_or(HEIGHT as f32, |rect| rect.origin.y.0 + rect.size.height.0);
    let tall = ((height * scale).ceil() as u32).min(picture.height());
    Ok(imageops::crop_imm(&picture, 0, 0, picture.width(), tall).to_image())
}

/// `pictures` side by side on a white ground, `gap` apart.
fn beside(pictures: &[RgbaImage], gap: u32) -> RgbaImage {
    let height = pictures.iter().map(RgbaImage::height).max().unwrap_or(1);
    let width = pictures.iter().map(RgbaImage::width).sum::<u32>()
        + gap * pictures.len().saturating_sub(1) as u32;
    let mut sheet = RgbaImage::from_pixel(width.max(1), height, Rgba([255, 255, 255, 255]));
    let mut x = 0i64;
    for picture in pictures {
        imageops::overlay(&mut sheet, picture, x, 0);
        x += i64::from(picture.width() + gap);
    }
    sheet
}

/// Where the progress page keeps the accent pictures.
fn progress_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/progress/shots/accent")
}

/// Write `picture` as `name` in `dir` and in the progress page's `shots/accent`.
fn keep(picture: &RgbaImage, dir: &Path, name: &str) -> Result<(), GalleryError> {
    for place in [dir.to_path_buf(), progress_dir()] {
        std::fs::create_dir_all(&place).map_err(|source| GalleryError::Write {
            path: place.clone(),
            source,
        })?;
        let path = place.join(name);
        picture
            .save(&path)
            .map_err(|source| GalleryError::Encode { path, source })?;
    }
    eprintln!("{}", dir.join(name).display());
    Ok(())
}

/// Render both sheets and every crop into `dir`.
pub fn run(dir: &Path) -> Result<(), GalleryError> {
    for scheme in Scheme::ALL {
        let columns = Column::ALL
            .into_iter()
            .map(|column| {
                let picture = column_at(column, scheme)?;
                let name = format!("accent-{}-{}.png", column.slug(), scheme.slug());
                keep(&picture, dir, &name)?;
                Ok(picture)
            })
            .collect::<Result<Vec<_>, GalleryError>>()?;
        let sheet = beside(&columns, GAP);
        keep(
            &sheet,
            dir,
            &format!("accent-candidates-{}.png", scheme.slug()),
        )?;
    }
    Ok(())
}
