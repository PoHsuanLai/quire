//! What a tile shows: its thumbnail over its caption, or the field the person is renaming it in.

use crate::components::content::icon_view::IconView;
use crate::components::lists::collection_view::item::{CollectionItem, Thumbnail};
use crate::components::lists::collection_view::size::CollectionSize;
use crate::focus::click::kept_click;
use dioxus::prelude::*;
use ds_core::text::clip::Truncate;
use ds_style::icon::render::{IconPx, IconSize};

/// The thumbnail's picture: an image fitted in its square, or an icon at the square's size.
fn picture(thumbnail: &Thumbnail, size: CollectionSize) -> Element {
    match thumbnail {
        Thumbnail::Image(source) => rsx! {
            img { class: "ds-cv-image", alt: "", src: "{source.0}", draggable: "false" }
        },
        Thumbnail::Icon(source) => rsx! {
            IconView {
                source: source.clone(),
                size: IconSize::Px(IconPx(size.thumb().0 as u8)),
            }
        },
    }
}

/// The caption line: the name cut from the middle to the size's character budget, one line (Blitz
/// cannot clamp a wrapped caption to two lines, so the tile spends its width on one).
fn caption(name: &str, size: CollectionSize) -> Element {
    let shown = Truncate::Middle.clip(name, size.caption_chars());
    rsx! {
        span { class: "ds-cv-caption", "{shown}" }
    }
}

/// The field in the caption's place. The keys, presses and drags inside it stay the field's, as
/// in a `Row` being edited, so the grid around it neither moves its cursor, jumps by type-select
/// nor opens on them.
fn edit_slot(slot: Element) -> Element {
    rsx! {
        span {
            class: "ds-cv-edit",
            onkeydown: |event| event.stop_propagation(),
            onkeyup: |event| event.stop_propagation(),
            onclick: |event| {
                event.stop_propagation();
                kept_click(&event);
            },
            ondoubleclick: |event| event.stop_propagation(),
            oncontextmenu: |event| event.stop_propagation(),
            onmousedown: |event| event.stop_propagation(),
            onpointerdown: |event| event.stop_propagation(),
            {slot}
        }
    }
}

/// `item`'s thumbnail and caption (or its field), drawn at `size`.
pub(crate) fn content<K>(item: &CollectionItem<K>, size: CollectionSize) -> Element {
    let words = match &item.edit {
        Some(editing) => edit_slot(editing.clone()),
        None => caption(&item.caption, size),
    };
    rsx! {
        span { class: "ds-cv-thumb", "aria-hidden": "true", {picture(&item.thumbnail, size)} }
        {words}
        if let Some(badge) = &item.badge {
            span { class: "ds-cv-badge", {badge.clone()} }
        }
    }
}
