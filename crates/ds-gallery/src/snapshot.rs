//! `--snapshot DIR`: every page in both schemes and two accents at the standard motion level,
//! rendered headless on the CPU (deterministic), written as PNGs with a contact sheet, and the
//! PNGs copied to the progress page's shots.

use crate::app::App;
use crate::axes::{Axes, Showcase, start_with};
use crate::error::GalleryError;
use crate::page::Page;
use crate::registry;
use crate::sheet;
use ds::{Accent, Motion, Scheme, Theme, Typeface};
use ds_native::{Viewport, snapshot_at};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The width every page is drawn at, in logical pixels.
pub const WIDTH: u32 = 1280;

/// The schemes a sheet sweeps.
pub const SCHEMES: [Scheme; 2] = [Scheme::Light, Scheme::Dark];

/// The accents a sheet sweeps: Postmark, and green (the brief's "pine"; quire's six accents have
/// no pine, and green is the nearest).
pub const ACCENTS: [Accent; 2] = [Accent::Postmark, Accent::Green];

/// When each picture is taken: long after every CSS entrance and transition has ended. Rust
/// timers do not run in a snapshot (ds_native::snapshot), so a picture is CSS time only.
const AT: Duration = Duration::from_secs(10);

/// One picture on the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shot {
    /// The page.
    pub page: Page,
    /// The scheme.
    pub scheme: Scheme,
    /// The accent.
    pub accent: Accent,
    /// The device scale in percent.
    pub scale: u16,
    /// The typeface.
    pub typeface: Typeface,
}

impl Shot {
    /// Its file name: `tokens-light-postmark.png`, with `-editorial` before the extension under
    /// the Editorial typeface.
    pub fn file(&self) -> String {
        let typeface = match self.typeface {
            Typeface::System => "",
            Typeface::Editorial => "-editorial",
        };
        format!(
            "{}-{}-{}{typeface}.png",
            self.page.slug(),
            self.scheme.slug(),
            self.accent.slug()
        )
    }

    /// The axes it is drawn with: overlays posed, standard motion, the default Space.
    pub fn axes(&self) -> Axes {
        let theme = match self.scheme {
            Scheme::Light => Theme::Light,
            Scheme::Dark => Theme::Dark,
        };
        Axes {
            page: self.page,
            theme,
            accent: self.accent,
            motion: Motion::Standard,
            showcase: Showcase::Posed,
            typeface: self.typeface,
            ..Axes::default()
        }
    }

    /// The viewport: the gallery's width, the page's own height, at the shot's scale.
    pub fn viewport(&self) -> Viewport {
        Viewport {
            width: WIDTH,
            height: registry::entry(self.page).height,
            scale_percent: self.scale,
        }
    }
}

/// Every picture, page by page.
pub fn shots() -> Vec<Shot> {
    shots_of(&Page::ALL)
}

/// The pictures of `pages`, page by page.
pub fn shots_of(pages: &[Page]) -> Vec<Shot> {
    pages
        .iter()
        .copied()
        .flat_map(|page| {
            SCHEMES.into_iter().flat_map(move |scheme| {
                ACCENTS.into_iter().map(move |accent| Shot {
                    page,
                    scheme,
                    accent,
                    scale: 100,
                    typeface: Typeface::System,
                })
            })
        })
        .collect()
}

/// Render `shot` to pixels.
pub fn render(shot: &Shot) -> Result<image::RgbaImage, GalleryError> {
    start_with(shot.axes());
    let mut frames =
        snapshot_at(App, shot.viewport(), &[AT]).map_err(|source| GalleryError::Render {
            name: shot.file(),
            source,
        })?;
    frames.pop().ok_or_else(|| GalleryError::Render {
        name: shot.file(),
        source: ds_native::NativeError::Renderer("no frame".into()),
    })
}

/// Where the progress page keeps the gallery's pictures.
pub fn progress_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/progress/shots/gallery")
}

/// Render every shot into `dir` (only `page`'s when one is named, at `scale` percent, in
/// `typeface`), write
/// `dir/index.html`, and copy the pictures to [`progress_dir`]. Returns the pictures written.
pub fn run(
    dir: &Path,
    page: Option<Page>,
    scale: u16,
    typeface: Typeface,
) -> Result<Vec<Shot>, GalleryError> {
    let made = |path: &Path| {
        let path = path.to_path_buf();
        move |source| GalleryError::Write { path, source }
    };
    std::fs::create_dir_all(dir).map_err(made(dir))?;
    let progress = progress_dir();
    std::fs::create_dir_all(&progress).map_err(made(&progress))?;
    let shots = match page {
        Some(page) => shots_of(&[page]),
        None => shots(),
    }
    .into_iter()
    .map(|shot| Shot {
        scale,
        typeface,
        ..shot
    })
    .collect::<Vec<_>>();
    for (done, shot) in shots.iter().enumerate() {
        let picture = render(shot)?;
        let path = dir.join(shot.file());
        picture.save(&path).map_err(|source| GalleryError::Encode {
            path: path.clone(),
            source,
        })?;
        let copy = progress.join(shot.file());
        std::fs::copy(&path, &copy).map_err(made(&copy))?;
        eprintln!("[{}/{}] {}", done + 1, shots.len(), path.display());
    }
    let index = dir.join("index.html");
    let html = sheet::html(dir, &shots).map_err(made(dir))?;
    std::fs::write(&index, html).map_err(made(&index))?;
    eprintln!("contact sheet: {}", index.display());
    Ok(shots)
}

#[cfg(test)]
mod tests {
    use super::{ACCENTS, SCHEMES, shots};
    use crate::page::Page;
    use std::collections::HashSet;

    #[test]
    fn the_sheet_sweeps_every_page_in_both_schemes_and_two_accents() {
        let shots = shots();
        assert_eq!(shots.len(), Page::ALL.len() * SCHEMES.len() * ACCENTS.len());
        let files: HashSet<String> = shots.iter().map(|shot| shot.file()).collect();
        assert_eq!(files.len(), shots.len(), "two shots share a file name");
        assert!(files.contains("motion-lab-dark-green.png"));
    }
}
