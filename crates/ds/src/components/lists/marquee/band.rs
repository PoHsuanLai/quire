//! The rubber band as it is drawn: a translucent rectangle in the accent tint, laid over the
//! scroller's content so it scrolls with it.

use crate::components::lists::marquee::hook::Marquee;
use dioxus::prelude::*;

/// The band while a drag is on, nothing otherwise. It is the first child of the `Scroller` it
/// draws in, inside a positioned layer of no height at the content's top: its containing box. It reads the drag itself, so
/// the pointer moving redraws this and not the view around it.
#[component]
pub(crate) fn MarqueeBand(marquee: Marquee) -> Element {
    let Some(rect) = marquee.rect() else {
        return rsx! {};
    };
    let (left, top) = (rect.left().0, rect.top().0);
    let (width, height) = (rect.size.width.0, rect.size.height.0);
    rsx! {
        div { class: "ds-marquee-layer",
            div {
                class: "ds-marquee",
                "aria-hidden": "true",
                style: "left:{left}px;top:{top}px;width:{width}px;height:{height}px",
            }
        }
    }
}
