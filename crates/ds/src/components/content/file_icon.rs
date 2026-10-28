//! FileIcon: the standard file icon, drawn (customization design section 6.1). A page with a
//! folded corner carries the kind's mark in the kind's colour and, when there is room, the
//! extension as a small bold label; a folder is the blue Mac folder, a special one with its glyph
//! embossed; a picture is the file's own thumbnail on a page frame; artwork (a chosen set's icon,
//! or the person's own) is drawn whole. The standard shapes are inline SVG and the marks are
//! quire's glyphs, so it scales from 16 to 512 px.
//!
//! The page stays light in the dark scheme, as the reference's does; the folder takes its dark
//! tones. Under `icons.style = monochrome` (`style: FileIconStyle::Monochrome(tint)`) the folder
//! and the marks take the tint. At 16 and 32 px the label is dropped and the mark grows.
//!
//! Markup: `span.ds-file-icon[data-choice][data-kind][data-size][data-label]` of the parts in
//! `file_icon_parts`. Which choice an app draws for a file is the resolver's job
//! (`FileIcons::of`), not this component's.

use crate::components::content::file_icon_parts::{artwork, folder, page, picture};
use crate::components::content::image_source::ImageSource;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};
use ds_style::icon::file_tint::FileIconStyle;
use ds_style::icon::render::IconSize;

/// What a file icon shows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FileIconChoice {
    /// A page with the kind's mark and, for a file with one, its extension.
    Standard {
        /// Which mark and colour.
        kind: FileKind,
        /// The extension label, when the file has one that fits.
        label: Option<ExtLabel>,
    },
    /// The Mac folder, plain or with a special folder's glyph.
    Folder(FolderKind),
    /// The file's own picture on a page frame.
    Picture(ImageSource),
    /// A finished icon from a set of file icons or from the person's own change: drawn whole in
    /// the square, uncropped and with no page frame. (`IconPick::Artwork` of the resolver.)
    Artwork(ImageSource),
}

/// The smallest side that draws the extension label; under it the mark takes the room.
pub const LABEL_MIN_PX: u8 = 48;

impl FileIconChoice {
    /// The `data-choice` word.
    fn slug(&self) -> &'static str {
        match self {
            FileIconChoice::Standard { .. } => "standard",
            FileIconChoice::Folder(_) => "folder",
            FileIconChoice::Picture(_) => "picture",
            FileIconChoice::Artwork(_) => "artwork",
        }
    }

    /// The `data-kind` word: the file kind, the folder kind, or none for a picture.
    fn kind_slug(&self) -> Option<&'static str> {
        match self {
            FileIconChoice::Standard { kind, .. } => Some(kind.slug()),
            FileIconChoice::Folder(kind) => Some(kind.slug()),
            FileIconChoice::Picture(_) | FileIconChoice::Artwork(_) => None,
        }
    }

    /// What a screen reader reads when the caller gave no name.
    fn name(&self) -> String {
        match self {
            FileIconChoice::Standard { kind, .. } => kind.label().to_owned(),
            FileIconChoice::Folder(FolderKind::Plain) => "Folder".to_owned(),
            FileIconChoice::Folder(FolderKind::Trash) => "Trash".to_owned(),
            FileIconChoice::Folder(kind) => format!("{} folder", kind.label()),
            FileIconChoice::Picture(_) => "Picture".to_owned(),
            FileIconChoice::Artwork(_) => "File".to_owned(),
        }
    }
}

/// `choice` drawn `size` square (16 px by default; any size from 16 to 512 works). `style` is the
/// icon style (`FileIconStyle::of(icons.style, Tint::space(dots))`). The accessible name is
/// `common.aria_label`, else the kind's plain name ("PDF document", "Folder").
#[component]
pub fn FileIcon(
    choice: FileIconChoice,
    #[props(default)] size: IconSize,
    #[props(default)] style: FileIconStyle,
    #[props(default)] common: Common,
) -> Element {
    let px = size.px();
    let fit = px >= LABEL_MIN_PX;
    let name = common.aria_label.clone().unwrap_or_else(|| choice.name());
    let data = common.data_attributes();
    let mounted = common.clone();
    let inline = format!("--ic-size:{px}px;{}", style.style_attribute());
    rsx! {
        span {
            id: common.id.clone(),
            class: common.class("ds-file-icon"),
            role: "img",
            "aria-label": name,
            "data-choice": choice.slug(),
            "data-kind": choice.kind_slug(),
            "data-size": "{px}",
            "data-label": if fit { "shown" } else { "hidden" },
            "data-icon-style": style.slug(),
            style: inline,
            onmounted: move |event| mounted.mounted(event),
            ..data,
            match &choice {
                FileIconChoice::Standard { kind, label } => page(*kind, label.as_ref(), px, fit),
                FileIconChoice::Folder(kind) => folder(*kind, px),
                FileIconChoice::Picture(source) => picture(source),
                FileIconChoice::Artwork(source) => artwork(source),
            }
        }
    }
}
