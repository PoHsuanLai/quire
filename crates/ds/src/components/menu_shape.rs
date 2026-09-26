//! How a shaped row draws (sill Q290, `row_shape`): a file's thumbnail in the tile and its folder
//! under its name, a clipboard entry's text in the code face or its picture, and either one's
//! time as trailing data. Split from `menu_item`, which calls these for a row whose shape is not
//! `Plain`.

use crate::components::row_shape::{ClipBody, RowShape, clip_box, clip_lines};
use dioxus::prelude::*;

/// A plain row's shape, for the entries that carry none.
pub(crate) const PLAIN: &RowShape = &RowShape::Plain;

/// The `data-shape` word, on a shaped row only (a plain row's markup is unchanged).
pub(crate) fn slug(shape: &RowShape) -> Option<&'static str> {
    match shape {
        RowShape::Plain => None,
        RowShape::File { .. } => Some("file"),
        RowShape::Clip {
            body: ClipBody::Text { .. },
            ..
        } => Some("clip-text"),
        RowShape::Clip {
            body: ClipBody::Image { .. },
            ..
        } => Some("clip-image"),
    }
}

/// A file's thumbnail as the tile, when it has one; `None` keeps the row's own tile.
pub(crate) fn thumb_tile(shape: &RowShape) -> Option<Element> {
    let RowShape::File {
        thumb: Some(thumb), ..
    } = shape
    else {
        return None;
    };
    Some(rsx! {
        span { class: "ds-menu-tile", "data-tile": "thumb",
            img { class: "ds-menu-thumb", alt: "", src: thumb.0.clone(), draggable: "false" }
        }
    })
}

/// The words column: the title and detail as given, or the shape's own.
pub(crate) fn words(shape: &RowShape, title: Element, detail: Option<Element>) -> Element {
    match shape {
        RowShape::Plain => rsx! {
            span {
                b { class: "ds-menu-title", {title} }
                if let Some(detail) = detail {
                    small { class: "ds-menu-detail ds-truncate", {detail} }
                }
            }
        },
        RowShape::File { location, .. } => rsx! {
            span {
                b { class: "ds-menu-title", {title} }
                small { class: "ds-menu-detail ds-truncate", "{location}" }
            }
        },
        RowShape::Clip {
            body: ClipBody::Text { excerpt, lines },
            ..
        } => rsx! {
            span {
                span {
                    class: "ds-menu-clip",
                    style: "--lines:{clip_lines(*lines)}",
                    "{excerpt}"
                }
            }
        },
        RowShape::Clip {
            body: ClipBody::Image { src, size },
            ..
        } => {
            let (width, height) = clip_box(*size);
            rsx! {
                span {
                    b { class: "ds-menu-title", {title} }
                    img {
                        class: "ds-menu-clip-image",
                        alt: "",
                        src: src.0.clone(),
                        draggable: "false",
                        style: "width:{width:.2}px;height:{height:.2}px",
                    }
                }
            }
        }
    }
}

/// The trail column: a shaped row's time before the row's own trail.
pub(crate) fn trail(shape: &RowShape, text: String) -> Element {
    let when = match shape {
        RowShape::Plain => None,
        RowShape::File { modified, .. } => Some(modified.clone()),
        RowShape::Clip { age, .. } => Some(age.clone()),
    };
    rsx! {
        span { class: "ds-menu-trail",
            if let Some(when) = when {
                span { class: "ds-menu-when", "{when}" }
            }
            "{text}"
        }
    }
}
