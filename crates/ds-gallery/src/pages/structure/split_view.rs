//! SplitView: a sidebar pane and the content, dragged, collapsed and reset; and a pane whose body
//! is an `EdgePeek`, which keeps its peek while folded.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::app::edge_peek::EdgePeek;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::SidebarSection;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_core::press::Press;
use ds_core::vocab::RowState;

/// The SplitView section.
#[component]
pub fn SplitViewSection() -> Element {
    let mut sidebar = use_signal(|| Shown::Visible);
    let mut saves = use_signal(|| 0usize);
    rsx! {
        Section { title: "SplitView", note: "NSSplitView: drag the divider (a 6 px zone with the hairline in it) to size the sidebar between 180 and 320; past half its least it folds away and a drag back opens it; double-click the divider to return to 240. The button folds it on a spring. The divider takes the arrow keys. The count below goes up once per finished drag or key adjustment, not per frame.",
            div { class: "g-row",
                span { class: "g-note", "Sizes saved: {saves()}" }
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
                    on_resized: move |_| saves += 1,
                    div { class: "g-split-main", "Content" }
                }
            }
        }
    }
}

/// The sidebar a peeking pane holds.
fn places(here: &'static str, onselect: EventHandler<&'static str>) -> Vec<ListItem<&'static str>> {
    [("inbox", "Inbox", Icon::Inbox), ("sent", "Sent", Icon::Send), ("trash", "Trash", Icon::Trash)]
        .into_iter()
        .map(|(key, title, icon)| {
            ListItem::row(
                key,
                title,
                rsx! {
                    Row {
                        title: TextLine::from(title),
                        leading: RowLeading::Icon(icon),
                        state: RowState { selection: Selection::of(&here, &key), ..RowState::default() },
                        onclick: move |_| onselect.call(key),
                    }
                },
            )
        })
        .collect()
}

/// A pane that hosts an `EdgePeek`.
#[component]
pub fn SplitViewPeekSection() -> Element {
    let mut pinned = use_signal(|| Shown::Hidden);
    let mut here = use_signal(|| "inbox");
    let onselect = EventHandler::new(move |key: &'static str| here.set(key));
    let body = rsx! {
        EdgePeek { label: "Sidebar", pinned: pinned(), onpin: move |()| pinned.set(Shown::Visible),
            Sidebar::<&'static str> {
                label: "Mail",
                cursor: Some(here()),
                onselect,
                sections: vec![SidebarSection::List(places(here(), onselect))],
            }
        }
    };
    rsx! {
        Section { title: "SplitView: a pane that peeks", note: "SplitPane::peeking: the pane's body is an EdgePeek. Folded away (drag the divider shut, or press the button), the pane stops clipping: point at the left edge to float the sidebar over the content, click the edge to pin it again.",
            div { class: "g-row",
                Button {
                    label: "Sidebar",
                    size: ControlSize::Small,
                    value: Some(if pinned() == Shown::Visible { Check::On } else { Check::Off }),
                    onclick: move |_: Press| pinned.set(pinned().flipped()),
                }
            }
            div { class: "g-stage g-split-stage",
                SplitView {
                    label: "Peeking example",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, body).shown(pinned()).peeking()],
                    on_shown: move |(_, shown)| pinned.set(shown),
                    div { class: "g-split-main", "Content" }
                }
            }
        }
    }
}
