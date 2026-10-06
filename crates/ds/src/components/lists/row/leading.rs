//! What leads a row: the glyph, picture, letter or avatar at its start (design/30 section 1.7,
//! row anatomy).

use crate::components::content::avatar::{AvatarFace, face};
use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::forms::icon_tile::{TileFace, tile};
use crate::components::lists::row::shape::RowShape;
use dioxus::prelude::*;
use ds_core::vocab::Selection;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// The element at a row's start.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RowLeading {
    /// Nothing: the title starts the row.
    #[default]
    None,
    /// A glyph in the secondary ink.
    Icon(Icon),
    /// A glyph on a disc: the paper disc, or the accent disc while `Selected` (the network or
    /// device in use).
    Disc(Icon, Selection),
    /// Any icon source: an app's icon file (drawn as it is, with no plate under it), a symbolic
    /// icon (in the text colour) or a glyph.
    Source(IconSource),
    /// A letter or two on a plate.
    Text(String),
    /// An avatar.
    Avatar(AvatarFace),
    /// A grouped row's tile: a colour with a white glyph, or the circular avatar.
    Tile(TileFace),
}

impl RowLeading {
    /// The `data-leading` word: which element leads.
    pub(crate) fn slug(&self) -> Option<&'static str> {
        match self {
            RowLeading::None => None,
            RowLeading::Icon(_) => Some("icon"),
            RowLeading::Disc(..) => Some("disc"),
            RowLeading::Source(IconSource::Image(_)) => Some("image"),
            RowLeading::Source(_) => Some("source"),
            RowLeading::Text(_) => Some("text"),
            RowLeading::Avatar(_) => Some("avatar"),
            RowLeading::Tile(face) => Some(face.slug()),
        }
    }
}

/// The row's `data-leading` word: a thumbnail replaces whatever leads, as it does in `draw`.
pub(crate) fn row_slug(leading: &RowLeading, shape: &RowShape) -> Option<&'static str> {
    shape
        .thumb_src()
        .map(|_| "thumb")
        .or_else(|| leading.slug())
}

/// The leading part drawn. A file row's thumbnail takes the place of whatever leads it.
pub(crate) fn draw(leading: &RowLeading, shape: &RowShape) -> Element {
    if let Some(thumb) = shape.thumb_src() {
        return rsx! {
            span { class: "ds-row-leading", "data-leading": "thumb",
                img { class: "ds-row-thumb", alt: "", src: thumb, draggable: "false" }
            }
        };
    }
    let Some(slug) = leading.slug() else {
        return rsx! {};
    };
    let inner = match leading {
        RowLeading::None => rsx! {},
        RowLeading::Icon(icon) => rsx! {
            Glyph { icon: *icon, size: IconSize::Base }
        },
        RowLeading::Disc(icon, _) => rsx! {
            Glyph { icon: *icon, size: IconSize::Base }
        },
        RowLeading::Source(source) => rsx! {
            IconView { source: source.clone(), size: IconSize::Tile }
        },
        RowLeading::Text(text) => rsx! { "{text}" },
        RowLeading::Avatar(avatar) => face(*avatar),
        RowLeading::Tile(face) => tile(*face),
    };
    let disc = match leading {
        RowLeading::Disc(_, Selection::Selected) => Some("on"),
        RowLeading::Disc(_, Selection::Unselected) => Some("off"),
        _ => None,
    };
    rsx! {
        span { class: "ds-row-leading", "data-leading": slug, "data-disc": disc, {inner} }
    }
}
