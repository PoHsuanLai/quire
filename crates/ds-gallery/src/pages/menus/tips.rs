//! A context menu that keeps its unavailable items dimmed, and a row's tip that says why.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;

/// The kept-and-tipped context menu section.
#[component]
pub fn KeptItemsAndTips() -> Element {
    let items = vec![
        MenuItem::new(1, "Copy"),
        MenuItem::new(2, "Stop")
            .with_availability(Availability::Disabled)
            .with_tip("Wait for the agent to finish"),
        MenuItem::new(3, "Paste"),
    ];
    rsx! {
        Section { title: "Context menu, items kept", note: "ContextUnavailable::Dim keeps an unavailable item dimmed instead of leaving it out; hover the dimmed row for its tip, shown at once.",
            div { class: "g-stage",
                div { class: "g-menu-card",
                    Menu::<u8> {
                        placement: MenuPlacement::Context,
                        anchor: Anchor::Point(Point::default()),
                        items,
                        unavailable: ContextUnavailable::Dim,
                        flow: Flow::Inline,
                        onpick: |_| {},
                        onclose: |_| {},
                    }
                }
            }
        }
    }
}
