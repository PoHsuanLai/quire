//! The parts of a `FileIcon`: the page, the folder and the picture frame. Each shape is one
//! `svg` filled with `currentColor`, coloured by the stylesheet (`file_icon.css`), so a scheme or
//! a tint changes a part's `color` and nothing in the markup. Sizes are written as attributes in
//! pixels (an `svg`'s width and height are presentation attributes in Blitz).

use crate::components::content::file_icon::LabelFit;
use crate::components::content::image_source::ImageSource;
use dioxus::prelude::*;
use ds_style::icon::Icon;
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};
use ds_style::icon::render::{Glyph, IconPx, IconSize};
use ds_style::icon::style::GlyphStyle;

/// The page's outline on a 48 x 64 grid (0.75 aspect): rounded corners and a cut top right,
/// 11 across (22 % of the width).
const PAGE: &str = "M3 0H37L48 11V61a3 3 0 0 1-3 3H3a3 3 0 0 1-3-3V3a3 3 0 0 1 3-3z";
/// The folded flap: the cut's triangle, turned down.
const FLAP: &str = "M37 0V8a3 3 0 0 0 3 3H48z";
/// The folder's back and tab on a 64 x 52 grid.
const FOLDER_BACK: &str = "M4 0H20.5a4 4 0 0 1 2.9 1.25L26.6 4.75A4 4 0 0 0 29.5 6H60a4 4 0 0 1 4 4V48a4 4 0 0 1-4 4H4a4 4 0 0 1-4-4V4a4 4 0 0 1 4-4z";
/// The folder's front.
const FOLDER_FRONT: &str =
    "M4 10H60a4 4 0 0 1 4 4V48a4 4 0 0 1-4 4H4a4 4 0 0 1-4-4V14a4 4 0 0 1 4-4z";

/// The page's box inside a square of `px`: 69 % across and 92 % down, the page's 0.75 aspect.
fn page_box(px: u8) -> (f32, f32) {
    (f32::from(px) * 0.69, f32::from(px) * 0.92)
}

/// The folder's box inside a square of `px`: 94 % across, the grid's 64:52 down.
fn folder_box(px: u8) -> (f32, f32) {
    let width = f32::from(px) * 0.94;
    (width, width * 52.0 / 64.0)
}

/// A glyph `side` px square, solid, for a mark.
fn mark_glyph(icon: Icon, side: f32) -> Element {
    let side = side.round().clamp(6.0, 255.0) as u8;
    rsx! { Glyph { icon, size: IconSize::Px(IconPx(side)), style: GlyphStyle::Solid } }
}

/// One shape layer: an `svg` `width` by `height` px holding `outline`, drawn by `class`.
fn layer(
    class: &'static str,
    grid: &'static str,
    (width, height): (f32, f32),
    outline: &'static str,
) -> Element {
    rsx! {
        svg {
            class: "ds-file-layer {class}",
            "data-ds-svg": "file-icon",
            width: "{width:.1}",
            height: "{height:.1}",
            view_box: grid,
            "aria-hidden": "true",
            path { d: outline, fill: "currentColor" }
        }
    }
}

/// The standard page: shadow, paper, fold, hairline edge, the kind's mark and the label.
pub(crate) fn page(kind: FileKind, label: Option<&ExtLabel>, px: u8, fit: LabelFit) -> Element {
    let size = page_box(px);
    let side = f32::from(px);
    let label = label.filter(|_| fit == LabelFit::Shown);
    let mark_side = match label {
        Some(_) => side * 0.30,
        None => side * 0.5,
    };
    // The label of a markless kind is the page's only mark, so it is larger and centred.
    let label_size = match kind {
        FileKind::Other => side * 0.2,
        _ => side * 0.15,
    };
    let slot = match label {
        Some(_) => "labelled",
        None => "bare",
    };
    rsx! {
        span { class: "ds-file-page", "data-slot": slot,
            span { class: "ds-file-shadow", "aria-hidden": "true" }
            {layer("ds-file-paper", "0 0 48 64", size, PAGE)}
            {layer("ds-file-flap", "0 0 48 64", size, FLAP)}
            svg {
                class: "ds-file-layer ds-file-edge",
                "data-ds-svg": "file-icon",
                width: "{size.0:.1}",
                height: "{size.1:.1}",
                view_box: "0 0 48 64",
                "aria-hidden": "true",
                path { d: PAGE, fill: "none", "stroke": "currentColor", "stroke-width": "0.5" }
            }
            if let Some(icon) = kind.mark() {
                span { class: "ds-file-mark", {mark_glyph(icon, mark_side)} }
            }
            if let Some(label) = label {
                span { class: "ds-file-label", style: "--file-label-size:{label_size:.1}px", "{label.as_str()}" }
            }
        }
    }
}

/// The folder: back and tab, front, and a special folder's glyph embossed on the front (a light
/// copy a pixel lower, the dark one over it).
pub(crate) fn folder(kind: FolderKind, px: u8) -> Element {
    let size = folder_box(px);
    let emboss = size.0 * 0.38;
    rsx! {
        span { class: "ds-file-folder",
            span { class: "ds-file-shadow", "aria-hidden": "true" }
            {layer("ds-file-tab", "0 0 64 52", size, FOLDER_BACK)}
            {layer("ds-file-body", "0 0 64 52", size, FOLDER_FRONT)}
            if let Some(icon) = kind.emboss() {
                span { class: "ds-file-emboss ds-file-emboss-light", {mark_glyph(icon, emboss)} }
                span { class: "ds-file-emboss", {mark_glyph(icon, emboss)} }
            }
        }
    }
}

/// The file's own picture, cropped to fill a page-shaped frame with the page's edge and shadow.
pub(crate) fn picture(source: &ImageSource) -> Element {
    rsx! {
        span { class: "ds-file-page ds-file-photo",
            span { class: "ds-file-shadow", "aria-hidden": "true" }
            img { class: "ds-file-picture", alt: "", src: "{source.0}", draggable: "false" }
        }
    }
}

/// A finished icon, whole: contained in the square, no frame and no crop.
pub(crate) fn artwork(source: &ImageSource) -> Element {
    rsx! {
        img { class: "ds-file-artwork", alt: "", src: "{source.0}", draggable: "false" }
    }
}
