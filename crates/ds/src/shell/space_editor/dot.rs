//! One Space's dot in the sidebar foot, the switch between Spaces (design/04-COMPONENTS.md
//! section 32, `S:147-150`).

use crate::core::vocab::{Check, Selection, Shortcut};
use crate::style::space::dot_paint::DotPaint;
use crate::style::space::frame_vars::FrameVars;
use dioxus::prelude::*;

/// One Space's dot in the sidebar foot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    here: Selection,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
) -> Element {
    let pressed = match here {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    };
    let keys = shortcut.glyphs();
    let paint = DotPaint::gradient(&frame.stops);
    rsx! {
        button {
            r#type: "button",
            class: "ds-space-dot",
            "aria-pressed": pressed.aria(),
            "aria-label": "{name} Space",
            title: "{name} ({keys})",
            "data-stops": paint.count(),
            style: paint.style_attr(),
            onclick: move |_| onclick.call(()),
        }
    }
}
