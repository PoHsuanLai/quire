//! SplitView: a sidebar pane and the content, dragged, collapsed and reset.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{Button, Check, ControlSize, PaneSpec, Press, Shown, SplitPane, SplitView};

/// The SplitView section.
#[component]
pub fn SplitViewSection() -> Element {
    let mut sidebar = use_signal(|| Shown::Visible);
    rsx! {
        Section { title: "SplitView", note: "NSSplitView: drag the divider (a 6 px zone with the hairline in it) to size the sidebar between 180 and 320; past half its least it folds away and a drag back opens it; double-click the divider to return to 240. The button folds it on a spring. The divider takes the arrow keys.",
            div { class: "g-row",
                Button {
                    label: "Sidebar",
                    size: ControlSize::Small,
                    value: Some(if sidebar() == Shown::Visible { Check::On } else { Check::Off }),
                    onclick: move |_: Press| sidebar.set(sidebar().flipped()),
                }
            }
            div { class: "g-stage g-split-stage",
                SplitView {
                    label: "Example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! {
                        div { class: "g-split-side", "Sidebar" }
                    }).shown(sidebar())],
                    on_shown: move |(_, shown)| sidebar.set(shown),
                    div { class: "g-split-main", "Content" }
                }
            }
        }
    }
}
