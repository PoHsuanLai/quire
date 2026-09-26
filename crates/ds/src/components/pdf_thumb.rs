//! PdfThumb: a PDF's first page as a thumbnail (design/04-COMPONENTS.md section 45; sill M9's
//! launcher preview, and design/20 section 2.4's Quick Look later).
//!
//! ds draws; it never reads a file or rasterises (it is renderer-free, `scripts/check-boundary.sh`).
//! The caller hands it the page as a [`PdfPage`]: a Blitz app uses `ds_native::PdfFileThumb`,
//! which takes a path, rasterises off the UI thread and caches, and feeds this component. The
//! page is a paper sheet (`--foreign-ground`, white in both schemes) with a hairline edge, fitted
//! into `size` at the page's own aspect and centred ([`sheet_rect`]). While the page is being read
//! nothing shows until [`PDF_THUMB_GRACE`] has passed, so a fast read never flashes a
//! placeholder; after it, the dimmed blank sheet (the pending look's still frame). A document
//! with no pages is a blank sheet; one that cannot be read, or is locked, is a file-type glyph on
//! an app-icon plate.

use crate::components::icon_view::IconView;
use crate::components::image_source::{ImageSize, ImageSource};
use crate::components::pdf_thumb_grace::{Grace, Reading, use_grace};
use crate::components::shot_frame::picture_style;
use crate::geometry::{Point, Px, Rect, Size};
use crate::icon::Icon;
use crate::icon::external::IconSource;
use crate::icon::family::PlateFamily;
use crate::icon::render::{IconPx, IconSize};
use dioxus::prelude::*;
use std::time::Duration;

/// How long a page may take to arrive before the pending look shows: design/26's
/// `PendingGrace` (400 ms, Rust-only, not following the motion level: it measures the read, not
/// motion).
pub const PDF_THUMB_GRACE: Duration = Duration::from_millis(400);

/// A sheet whose page is not known yet (loading, or no pages): A4 portrait, in points.
pub const PDF_DEFAULT_SHEET: ImageSize = ImageSize {
    width: 595,
    height: 842,
};

/// What a thumbnail shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PdfPage {
    /// Being read: nothing, then the pending look once [`PDF_THUMB_GRACE`] has passed.
    #[default]
    Loading,
    /// The first page, rasterised.
    Ready {
        /// The page's pixels, drawn to fill the sheet.
        image: ImageSource,
        /// The page's displayed size (points, after rotation); only its ratio is read.
        sheet: ImageSize,
    },
    /// A document with no pages: a blank sheet.
    Empty,
    /// It could not be drawn: a glyph plate instead of a sheet.
    Failed(PdfTrouble),
}

/// Why a page could not be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfTrouble {
    /// Not a PDF, damaged past recovery, unreadable, or a page the rasteriser refused.
    Unreadable,
    /// Encrypted with a password that is not the empty one.
    Locked,
}

impl PdfTrouble {
    /// The `data-trouble` word.
    pub fn slug(self) -> &'static str {
        match self {
            PdfTrouble::Unreadable => "unreadable",
            PdfTrouble::Locked => "locked",
        }
    }

    fn glyph(self) -> Icon {
        match self {
            PdfTrouble::Unreadable => Icon::File,
            PdfTrouble::Locked => Icon::Lock,
        }
    }

    fn label(self) -> &'static str {
        match self {
            PdfTrouble::Unreadable => "PDF, no preview",
            PdfTrouble::Locked => "Locked PDF",
        }
    }
}

impl PdfPage {
    /// The `data-state` word, the grace deciding between `loading` and `pending`.
    fn slug(&self, grace: Grace) -> &'static str {
        match (self, grace) {
            (PdfPage::Loading, Grace::Within) => "loading",
            (PdfPage::Loading, Grace::Over) => "pending",
            (PdfPage::Ready { .. }, _) => "ready",
            (PdfPage::Empty, _) => "empty",
            (PdfPage::Failed(_), _) => "failed",
        }
    }

    /// The sheet's shape: the page's own, else A4 portrait.
    fn sheet(&self) -> ImageSize {
        match self {
            PdfPage::Ready { sheet, .. } => *sheet,
            PdfPage::Loading | PdfPage::Empty | PdfPage::Failed(_) => PDF_DEFAULT_SHEET,
        }
    }
}

/// A PDF's first page, fitted into `size` at its own aspect on a paper sheet. `label` is what a
/// screen reader reads for it (the file's name, say); "PDF preview" by default.
#[component]
pub fn PdfThumb(page: PdfPage, size: Size, #[props(default)] label: Option<String>) -> Element {
    let reading = match page {
        PdfPage::Loading => Reading::Yes,
        PdfPage::Ready { .. } | PdfPage::Empty | PdfPage::Failed(_) => Reading::No,
    };
    let grace = use_grace(reading);
    let state = page.slug(grace);
    let room = format!("width:{}px;height:{}px", size.width.0, size.height.0);
    let label = match (&page, label) {
        (PdfPage::Failed(trouble), None) => trouble.label().to_owned(),
        (_, Some(label)) => label,
        (_, None) => "PDF preview".to_owned(),
    };
    let busy = matches!(page, PdfPage::Loading).then_some("true");
    rsx! {
        div {
            class: "ds-pdf-thumb",
            "data-state": state,
            role: "img",
            "aria-label": label,
            "aria-busy": busy,
            style: room,
            match page {
                PdfPage::Failed(trouble) => plate(trouble, size),
                PdfPage::Ready { ref image, .. } => sheet(size, page.sheet(), Some(image.clone())),
                PdfPage::Loading | PdfPage::Empty => sheet(size, page.sheet(), None),
            }
        }
    }
}

/// The paper sheet, with the page's pixels when there are any.
fn sheet(room: Size, shape: ImageSize, image: Option<ImageSource>) -> Element {
    let style = picture_style(sheet_rect(room, shape));
    rsx! {
        div { class: "ds-pdf-thumb-sheet", style,
            if let Some(image) = image {
                img { class: "ds-pdf-thumb-page", alt: "", src: image.0, draggable: "false" }
            }
        }
    }
}

/// The file-type glyph on a red plate, two fifths of the room's shorter side (16 to 96 px).
fn plate(trouble: PdfTrouble, room: Size) -> Element {
    let side = (room.width.0.min(room.height.0) * 0.4)
        .clamp(16.0, 96.0)
        .round() as u8;
    rsx! {
        div { class: "ds-pdf-thumb-plate", "data-trouble": trouble.slug(),
            IconView {
                source: IconSource::Glyph(trouble.glyph()),
                size: IconSize::Px(IconPx(side)),
                plate: Some(PlateFamily::Red),
            }
        }
    }
}

/// The largest rect of `shape`'s ratio inside `room`, centred. A shape with no area fills it.
pub fn sheet_rect(room: Size, shape: ImageSize) -> Rect {
    let size = match (shape.width, shape.height) {
        (0, _) | (_, 0) => room,
        (width, height) => {
            let ratio = height as f32 / width as f32;
            if room.width.0 * ratio <= room.height.0 {
                Size {
                    width: room.width,
                    height: Px(room.width.0 * ratio),
                }
            } else {
                Size {
                    width: Px(room.height.0 / ratio),
                    height: room.height,
                }
            }
        }
    };
    Rect {
        origin: Point {
            x: Px((room.width.0 - size.width.0) / 2.0),
            y: Px((room.height.0 - size.height.0) / 2.0),
        },
        size,
    }
}

#[cfg(test)]
mod tests {
    use super::{PDF_DEFAULT_SHEET, sheet_rect};
    use crate::components::image_source::ImageSize;
    use crate::geometry::{Px, Size};

    #[test]
    fn the_sheet_keeps_the_page_aspect_centred_in_the_room() {
        const ROOM: Size = Size {
            width: Px(200.0),
            height: Px(200.0),
        };
        // (page, left, top, width, height)
        #[rustfmt::skip]
        let cases: &[(ImageSize, f32, f32, f32, f32)] = &[
            (ImageSize { width: 612, height: 792 }, 22.73, 0.0, 154.55, 200.0),
            (ImageSize { width: 842, height: 595 }, 0.0, 29.33, 200.0, 141.33),
            (ImageSize { width: 100, height: 100 }, 0.0, 0.0, 200.0, 200.0),
            (ImageSize { width: 0, height: 100 }, 0.0, 0.0, 200.0, 200.0),
            (PDF_DEFAULT_SHEET, 29.33, 0.0, 141.33, 200.0),
        ];
        for &(page, left, top, width, height) in cases {
            let rect = sheet_rect(ROOM, page);
            let got = [
                rect.left().0,
                rect.top().0,
                rect.size.width.0,
                rect.size.height.0,
            ];
            for (g, want) in got.iter().zip([left, top, width, height]) {
                assert!((g - want).abs() < 0.01, "{page:?}: {got:?}");
            }
        }
    }
}
