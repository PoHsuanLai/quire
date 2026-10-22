//! Picture: an image in a rounded frame, cropped to fill it (`object-fit: cover`), as Finder's
//! Quick Look shows a wallpaper or a photo. The frame is as wide as its parent and takes the
//! shape's ratio ([`PictureShape::Landscape`] is the 16:10 of a screen); the picture is cropped
//! to it and never stretched. The corner radius is `--r-media`.
//!
//! The alt text is required: it is the picture's accessible name. quire never fetches, so the
//! source is an [`ImageSource`] (a `data:` URI or a `file:` URL).
//!
//! Markup: `div.ds-picture[data-shape]` of `img.ds-picture-image`.

use crate::components::content::image_source::ImageSource;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;

/// The frame's ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PictureShape {
    /// 16 wide to 10 tall, a screen's.
    #[default]
    Landscape,
    /// As tall as wide.
    Square,
}

/// `source` cropped to fill a rounded frame of `shape`. `alt` says what the picture shows.
#[component]
pub fn Picture(
    source: ImageSource,
    alt: String,
    #[props(default)] shape: PictureShape,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    let mounted = common.clone();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-picture"),
            "data-shape": shape.slug(),
            onmounted: move |event| mounted.mounted(event),
            ..data,
            img {
                class: "ds-picture-image",
                alt,
                src: "{source.0}",
                draggable: "false",
            }
        }
    }
}
