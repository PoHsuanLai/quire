//! EdgeStrip: with the sidebar hidden, a strip on the left edge brings it back as a floating
//! panel (design/04-COMPONENTS.md section 33).

use dioxus::prelude::*;

/// The reveal edge. Render it only while the sidebar is hidden; the pointer entering it asks
/// for the peek (design/06-INTERACTIONS.md section 7).
#[component]
pub fn EdgeStrip(onenter: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "ds-edge",
            "aria-hidden": "true",
            onpointerenter: move |_| onenter.call(()),
        }
    }
}
