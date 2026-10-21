//! Status bar: the thin row along the bottom of a window's content (Finder's "12 items, 40 GB
//! available"). A leading slot, a centred line of text in the secondary ink, a trailing slot,
//! and a hairline on top. The words are the app's (`status_bar_model::item_count_text` writes the
//! usual count); a bottom bar is for small status, never for anything critical (design/27
//! section 5.10).
//!
//! Markup: `div.ds-status-bar[role=status]` of `span.ds-status-bar-leading`,
//! `span.ds-status-bar-text` and `span.ds-status-bar-trailing`. The two slots are always
//! present, so the text stays centred when only one of them is used.

use crate::root::common::Common;
use dioxus::prelude::*;

/// A status bar showing `text`, with `leading` before it and `trailing` after it (a zoom
/// slider, a view switch).
#[component]
pub fn StatusBar(
    text: String,
    #[props(default)] leading: Option<Element>,
    #[props(default)] trailing: Option<Element>,
    #[props(default)] common: Common,
) -> Element {
    let mounted = common.clone();
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-status-bar"),
            role: "status",
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| mounted.mounted(event),
            ..data,
            span { class: "ds-status-bar-leading",
                if let Some(leading) = leading {
                    {leading}
                }
            }
            span { class: "ds-status-bar-text", "{text}" }
            span { class: "ds-status-bar-trailing",
                if let Some(trailing) = trailing {
                    {trailing}
                }
            }
        }
    }
}
