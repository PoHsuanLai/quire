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

/// How a [`SpaceDot`] looks. `Swatch` is the Space's colour (the editor); `Foot` is the small
/// quiet dot of the sidebar foot: ink when current, faint ink otherwise, no colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DotFace {
    #[default]
    Swatch,
    Foot,
}

impl DotFace {
    /// The `data-face` value, set on the foot face only (the swatch keeps its old markup).
    pub fn slug(self) -> Option<&'static str> {
        match self {
            DotFace::Swatch => None,
            DotFace::Foot => Some("foot"),
        }
    }
}

/// One Space's dot: the editor's colour swatch, or with `face: DotFace::Foot` the foot's dot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    selection: Selection,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
    #[props(default)] face: DotFace,
    #[props(default)] common: Common,
) -> Element {
    let pressed = match selection {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    };
    let paint = match face {
        DotFace::Swatch => Some(DotPaint::gradient(&frame.stops)),
        DotFace::Foot => None,
    };
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
            "data-face": face.slug(),
            "data-stops": paint.as_ref().map(DotPaint::count),
            "data-selected": selection.slug(),
            style: paint.as_ref().map(DotPaint::style_attr),
            onclick: move |_| onclick.call(()),
            onmounted: {
                let tip = tip.clone();
                move |event| {
                    tip.mounted(&event);
                    common.mounted(event);
                }
            },
            ..data,
            if face == DotFace::Foot {
                span { class: "ds-space-pip" }
            }
        }
        {tip.surface()}
    }
}
