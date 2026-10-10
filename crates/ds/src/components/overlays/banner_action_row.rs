//! BannerActionRow: the row a banner lays out when it asks for something and takes the answer
//! in place: a text field that takes the width left, then the buttons that act on it ("Allow",
//! "Cancel"). It is not the window's toolbar band (`chrome::Toolbar`); it is the banner's own
//! action line, put in `InlineBanner`'s `actions` or under any banner body.
//!
//! Markup: `div.ds-banner-action-row[role=group]` of `div.ds-banner-action-row-field` (the
//! `field`) and `div.ds-banner-action-row-buttons` (the `children`).

use crate::root::common::Common;
use dioxus::prelude::*;

/// A banner's action row: `field` takes the room that is left, `children` (the buttons) keep
/// their size at the end. `label` names the group.
#[component]
pub fn BannerActionRow(
    #[props(into)] label: String,
    field: Element,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-banner-action-row"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            onmounted: move |event| common.mounted(event),
            ..data,
            div { class: "ds-banner-action-row-field", {field} }
            div { class: "ds-banner-action-row-buttons", {children} }
        }
    }
}
