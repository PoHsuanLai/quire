//! Status bar: the thin row at the bottom of a window's content, with its text alone, with the
//! count helper's two wordings, and with a control in each slot.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::prelude::*;

/// The Status bar section.
#[component]
pub fn StatusBarSection() -> Element {
    rsx! {
        Section { title: "Status bar", note: "Finder's bottom bar: small secondary text centred between two slots, over a hairline. item_count_text writes the usual count (one item, many items, the room left when the app says it); the slots hold small controls and the text stays centred when only one is used.",
            div { class: "g-list", style: "width:520px",
                StatusBar { text: item_count_text(12, Some("40 GB")) }
            }
            div { class: "g-list", style: "width:520px",
                StatusBar { text: item_count_text(1, None) }
            }
            div { class: "g-list", style: "width:520px",
                StatusBar {
                    text: item_count_text(3, Some("1.2 TB")),
                    leading: rsx! {
                        span { "Downloads" }
                    },
                    trailing: rsx! {
                        span { "Icons" }
                    },
                }
            }
        }
    }
}
