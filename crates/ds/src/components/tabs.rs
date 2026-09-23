//! Tabs: switch between pages of one surface, with a spring underline
//! (design/04-COMPONENTS.md section 12).

use crate::components::vocab::Selection;
use dioxus::prelude::*;

/// A tab bar.
#[component]
pub fn Tabs<T: Clone + PartialEq + 'static>(
    label: String,
    tabs: Vec<(T, String)>,
    value: T,
    onchange: EventHandler<T>,
) -> Element {
    // The underline is a real span, not `::after`: pseudo-elements are unverified in Blitz and
    // stylo keys animations to non-pseudo nodes (O-22's fallback).
    // `aria-controls` is in the doc's markup, but the frozen props carry no panel id.
    rsx! {
        div {
            class: "ds-tabs",
            role: "tablist",
            "aria-label": "{label}",
            for (tab, text) in tabs {
                button {
                    r#type: "button",
                    class: "ds-tab",
                    role: "tab",
                    "aria-selected": Selection::of(&tab, &value).aria(),
                    onclick: move |_| onchange.call(tab.clone()),
                    "{text}"
                    span { class: "ds-tab-underline" }
                }
            }
        }
    }
}
