//! IconButtonGroup: a row of small icon buttons that share one rounded group, as in a block or
//! inline editor's toolbar. The caller's buttons are `Button`s with `Bezel::Toolbar` and
//! `ImagePosition::Only` at the group's `size`; the group draws the shared plate and keeps them
//! a hairline apart. It is a layout and a name for assistive technology, not a control: each
//! button keeps its own press, tip and state.
//!
//! Markup: `div.ds-icon-button-group[role=group][data-size]` of `children`.

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// A group of icon buttons named `label`, its plate rounded for `size` (Small unless given).
#[component]
pub fn IconButtonGroup(
    #[props(into)] label: String,
    children: Element,
    #[props(default = ControlSize::Small)] size: ControlSize,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-icon-button-group"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-size": size.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
