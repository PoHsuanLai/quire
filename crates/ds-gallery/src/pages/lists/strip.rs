//! Lists, for mail: a strip whose press acts before any measurement.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction};
use ds::components::app::row_more::RowMore;
use ds::components::app::thread_row::ThreadRow;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::prelude::*;
use ds_core::vocab::RowState;

/// Archive and snooze, each doing nothing on its measured click: the press says what happened.
fn actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

/// A row whose strip's press is heard at once.
#[component]
pub fn StripPress() -> Element {
    let mut pressed = use_signal(|| None::<ActionId>);
    let said = pressed().map_or("nothing yet".to_string(), |id| id.0);
    rsx! {
        Section {
            title: "A strip that acts before it measures",
            note: "on_press hears the button inside the click, before the rect read; the measured onclick follows only where layout answers.",
            div { class: "g-list g-stage-pad",
                List::<u8> {
                    label: "Threads",
                    items: vec![ListItem::row(
                        0,
                        "Re: UIDL stability",
                        rsx! {
                            ThreadRow {
                                state: RowState { selection: Selection::Selected, emphasis: Emphasis::Strong, ..RowState::default() },
                                name: "Dana Okafor",
                                via: None,
                                subject: "Re: UIDL stability",
                                snippet: None,
                                time: "09:41",
                                tags: rsx! {},
                                star: None,
                                strip: rsx! {
                                    HoverStrip {
                                        actions: actions(),
                                        shown: Shown::Visible,
                                        on_press: move |id| pressed.set(Some(id)),
                                    }
                                },
                                onclick: |_| {},
                            }
                        },
                    )],
                }
            }
            p { class: "g-note", "Pressed: {said}" }
        }
    }
}

/// One card row: the quiet "More actions" button in its tail, hidden, revealed by the caller, or
/// holding its menu open. The 2nd and 3rd rows show it with no pointer; hover the 1st to reveal it.
fn more_row(key: u8, subject: &'static str, tag: &'static str, more: Element) -> ListItem<u8> {
    ListItem::row(
        key,
        subject,
        rsx! {
            ThreadRow {
                state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Strong, ..RowState::default() },
                name: "Dana Okafor",
                via: None,
                subject,
                snippet: "Treat UIDL as stable only while UIDVALIDITY holds.".to_string(),
                time: "09:41",
                tags: rsx! { Chip { variant: ChipVariant::Accent, text: tag } },
                star: None,
                more,
                onclick: |_| {},
            }
        },
    )
}

/// The `more` slot: a quiet icon-only button in the tail, in flow beside the tags, never over
/// the time.
#[component]
pub fn MoreButton() -> Element {
    let mut open = use_signal(|| Shown::Visible);
    rsx! {
        Section {
            title: "Row: the quiet more button",
            note: "ThreadRow's `more` slot: a 26 px icon-only RowMore in flow in the tail, no pill, border or shadow, hidden until the row is hovered or focused. Row 2 is shown by its caller; row 3 holds its menu open (click it to toggle).",
            div { class: "g-list g-stage-pad",
                List::<u8> {
                    label: "More",
                    items: vec![
                        more_row(0, "Hover this row", "spec", rsx! { RowMore { onclick: |_| {} } }),
                        more_row(1, "Shown by its caller", "spec", rsx! { RowMore { shown: Shown::Visible, onclick: |_| {} } }),
                        more_row(
                            2,
                            "Menu open",
                            "spec",
                            rsx! { RowMore { expanded: open(), onclick: move |_| open.set(open().flipped()) } },
                        ),
                    ],
                }
            }
        }
    }
}
