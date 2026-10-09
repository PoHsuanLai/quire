//! One Space's dot in the sidebar foot, the switch between Spaces (design/04-COMPONENTS.md
//! section 32, `S:147-150`).

use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::use_tip;
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
    let paint = DotPaint::gradient(&frame.stops);
    let class = common.class("ds-space-dot");
    let data = common.data_attributes();
    let tip = use_tip(Some(TipText::new(name.clone()).with_shortcut(shortcut)));
    rsx! {
        button {
            r#type: "button",
            class,
            id: common.id.clone(),
            "aria-pressed": pressed.aria(),
            "aria-label": common.aria_label.clone().unwrap_or_else(|| format!("{name} Space")),
            title: tip.native(),
            onmouseover: {
                let tip = tip.clone();
                move |event| tip.over(&event)
            },
            onmouseleave: {
                let tip = tip.clone();
                move |_| tip.out()
            },
            onpointerdown: {
                let tip = tip.clone();
                move |_| tip.press()
            },
            "data-stops": paint.count(),
            "data-selected": selection.slug(),
            style: paint.style_attr(),
            onclick: move |_| onclick.call(()),
            onmounted: {
                let tip = tip.clone();
                move |event| {
                    tip.mounted(&event);
                    common.mounted(event);
                }
            },
            ..data,
        }
        {tip.surface()}
    }
}
