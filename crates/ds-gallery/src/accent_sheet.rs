//! `--accent-sheet DIR`: the settled accent (band B, Airy; design/03-COLOR.md section 20) on
//! every surface it paints, light and dark, over the calm wallpaper and a material: the primary
//! button, a toggle that is on, the segmented control, a chip, a link, the focus ring, a menu's
//! highlight, a selected row, the control center's tiles, the compact calendar (month title,
//! today disc) and the launcher's selected row; under them the six built-in hues. Writes
//! `accent-b-final-{light,dark}.png` in DIR and in the progress page's `shots/accent`.

use crate::accent_specimens::Specimens;
use crate::error::GalleryError;
use crate::style;
use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Scheme, Theme};
use ds_native::{Harness, Viewport};
use image::{RgbaImage, imageops};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A column's width in logical pixels.
const WIDTH: u32 = 380;
/// Tall enough for a column; the picture is cropped to it.
const HEIGHT: u32 = 1500;
/// The device scale: 2x, so the rings and hairlines read.
const SCALE: u16 = 200;

thread_local! {
    /// The scheme [`Sheet`] draws.
    static SCHEME: Cell<Scheme> = const { Cell::new(Scheme::Light) };
}

/// One column, rendered on its own (Blitz's CPU renderer loses layers in one large document).
#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Sheet() -> Element {
    let scheme = SCHEME.get();
    let theme = match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    };
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window,
            style { {style::CSS} }
            Specimens { scheme }
        }
    }
}

/// The sheet in `scheme`, cropped to its content.
fn sheet_at(scheme: Scheme) -> Result<RgbaImage, GalleryError> {
    SCHEME.set(scheme);
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
        scale_percent: SCALE,
    };
    let mut harness = Harness::new(Sheet, viewport);
    harness.advance(Duration::from_secs(2));
    let sheet = harness.rect(".g-acc");
    let picture = harness.render().map_err(|source| GalleryError::Render {
        name: format!("accent {}", scheme.slug()),
        source,
    })?;
    let scale = f32::from(SCALE) / 100.0;
    let height = sheet.map_or(HEIGHT as f32, |rect| rect.origin.y.0 + rect.size.height.0);
    let tall = ((height * scale).ceil() as u32).min(picture.height());
    Ok(imageops::crop_imm(&picture, 0, 0, picture.width(), tall).to_image())
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

/// Render both schemes into `dir`.
pub fn run(dir: &Path) -> Result<(), GalleryError> {
    for scheme in Scheme::ALL {
        let picture = sheet_at(scheme)?;
        keep(
            &picture,
            dir,
            &format!("accent-b-final-{}.png", scheme.slug()),
        )?;
    }
    Ok(())
}
