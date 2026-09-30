//! How a shaped row draws: a file's folder under its name, a clipboard entry's text in the code
//! face or its picture, and either one's time as trailing data. Called by `Row` for a row whose
//! shape is not `Plain`.

use crate::components::lists::row::shape::{ClipBody, Expiry, RowShape, clip_box, clip_lines};
use dioxus::prelude::*;

/// The `data-shape` word, on a shaped row only (a plain row's markup is unchanged).
pub(crate) fn slug(shape: &RowShape) -> Option<&'static str> {
    match shape {
        RowShape::Plain => None,
        RowShape::File { .. } => Some("file"),
        RowShape::Today {
            expiry: Expiry::Later,
            ..
        } => Some("today"),
        RowShape::Today {
            expiry: Expiry::Soon,
            ..
        } => Some("today-soon"),
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

/// The words column: the title and detail as given, or the shape's own.
pub(crate) fn words(shape: &RowShape, title: Element, detail: Option<Element>) -> Element {
    match shape {
        RowShape::Plain | RowShape::Today { .. } => rsx! {
            span { class: "ds-row-words",
                b { class: "ds-row-title ds-truncate", {title} }
                if let Some(detail) = detail {
                    small { class: "ds-row-detail ds-truncate", {detail} }
                }
            }
        },
        RowShape::File { location, .. } => rsx! {
            span { class: "ds-row-words",
                b { class: "ds-row-title ds-truncate", {title} }
                small { class: "ds-row-detail ds-truncate", "{location}" }
            }
        },
        RowShape::Clip {
            body: ClipBody::Text { excerpt, lines },
            ..
        } => rsx! {
            span { class: "ds-row-words",
                span {
                    class: "ds-row-clip",
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
                span { class: "ds-row-words",
                    b { class: "ds-row-title ds-truncate", {title} }
                    img {
                        class: "ds-row-clip-image",
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

/// A shaped row's time, drawn before the row's own accessory in a box of its own (Blitz drops a
/// plain inline span's margin).
pub(crate) fn when(shape: &RowShape) -> Option<Element> {
    let when = match shape {
        RowShape::Plain => return None,
        RowShape::File { modified, .. } => modified,
        RowShape::Today { left, .. } => left,
        RowShape::Clip { age, .. } => age,
    };
    Some(rsx! {
        span { class: "ds-row-when", "{when}" }
    })
}
