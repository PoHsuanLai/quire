//! One Space's dot in the sidebar foot, the switch between Spaces (design/04-COMPONENTS.md
//! section 32, `S:147-150`).

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Check, Selection, Shortcut};
use ds_core::word::Word;
use ds_style::space::dot_paint::DotPaint;
use ds_style::space::frame_vars::FrameVars;

/// One Space's dot in the sidebar foot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    selection: Selection,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
    #[props(default)] common: Common,
) -> Element {
    let pressed = match selection {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    };
    let keys = shortcut.glyphs();
    let paint = DotPaint::gradient(&frame.stops);
    let class = common.class("ds-space-dot");
    let data = common.data_attributes();
    rsx! {
        button {
            r#type: "button",
            class,
            id: common.id.clone(),
            "aria-pressed": pressed.aria(),
            "aria-label": common.aria_label.clone().unwrap_or_else(|| format!("{name} Space")),
            title: "{name} ({keys})",
            "data-stops": paint.count(),
            "data-selected": selection.slug(),
            style: paint.style_attr(),
            onclick: move |_| onclick.call(()),
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}
