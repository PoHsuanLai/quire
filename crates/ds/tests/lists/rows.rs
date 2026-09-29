//! The row cases: ListRow in every weight, star and motion state, and AnimatedList, with the
//! helpers that draw a thread row the way a consumer does.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{
    ActionId, AnimatedList, Chip, ChipVariant, Exit, Heal, HoverStrip, Icon, ListRow, MarkProvider,
    MarkSize, MarkStyle, Presence, ProviderMark, Px, RowState, StripAction,
};
use ds::{Check, DropState, Emphasis, Selection};

/// The four strip actions of the Spaces prototype (`S:1286-1288`).
pub fn strip_actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
        ("label", Icon::Tag, "Label", "Label…"),
        ("read", Icon::MailOpen, "Mark read", "Mark read"),
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

/// A thread row in `presence`, unread or read, with the via, tags and strip filled.
pub fn row(presence: Presence, emphasis: Emphasis, selection: Selection, star: Check) -> Element {
    row_in_drag(presence, emphasis, selection, star, DropState::Idle)
}

/// A present read row playing `drop` in a drag.
fn dragged_row(drop: DropState) -> Element {
    row_in_drag(
        Presence::Present,
        Emphasis::Plain,
        Selection::Unselected,
        Check::Off,
        drop,
    )
}

/// [`row`], playing `drop` in a drag.
fn row_in_drag(
    presence: Presence,
    emphasis: Emphasis,
    selection: Selection,
    star: Check,
    drop: DropState,
) -> Element {
    rsx! {
        ListRow {
            state: RowState { selection, emphasis, drop, ..RowState::default() },
            presence,
            name: "Dana Okafor",
            via: rsx! {
                ProviderMark { provider: MarkProvider::Google, size: MarkSize::Row, style: MarkStyle::Letter }
                "gmail"
            },
            subject: "Re: UIDL stability across servers",
            snippet: "Treat UIDL as stable only while UIDVALIDITY holds.".to_string(),
            time: "09:41",
            tags: rsx! { Chip { variant: ChipVariant::Accent, text: "spec" } },
            star: (star, EventHandler::new(|_| {})),
            strip: rsx! { HoverStrip { actions: strip_actions() } },
            onclick: |_| {},
        }
    }
}

/// A read, unselected, unstarred row: what a roster draws per entry, keyed by the consumer.
#[component]
pub fn Row(presence: Presence, heal: Option<Heal>, emphasis: Emphasis) -> Element {
    match heal {
        Some(heal) => healed_row(heal, emphasis),
        None => row(presence, emphasis, Selection::Unselected, Check::Off),
    }
}

fn plain_row(presence: Presence, emphasis: Emphasis) -> Element {
    row(presence, emphasis, Selection::Unselected, Check::Off)
}

/// A read row sliding into a gap by `heal`.
fn healed_row(heal: Heal, emphasis: Emphasis) -> Element {
    rsx! {
        ListRow {
            state: RowState { selection: Selection::Unselected, emphasis, ..RowState::default() },
            presence: Presence::Present,
            heal: Some(heal),
            name: "Dana Okafor",
            via: rsx! {
                ProviderMark { provider: MarkProvider::Google, size: MarkSize::Row, style: MarkStyle::Letter }
                "gmail"
            },
            subject: "Re: UIDL stability across servers",
            snippet: "Treat UIDL as stable only while UIDVALIDITY holds.".to_string(),
            time: "09:41",
            tags: rsx! { Chip { variant: ChipVariant::Accent, text: "spec" } },
            star: (Check::Off, EventHandler::new(|_| {})),
            strip: rsx! { HoverStrip { actions: strip_actions() } },
            onclick: |_| {},
        }
    }
}

/// A row 79 px below its place.
fn healing_row() -> Element {
    healed_row(Heal { dy: Px(79.0) }, Emphasis::Plain)
}

pub const ROW_CASES: &[Case] = &[
    // ListRow: weight, selection, star, and every motion state.
    Case {
        component: "list_row",
        state: "unread",
        make: || plain_row(Presence::Present, Emphasis::Strong),
    },
    Case {
        component: "list_row",
        state: "read",
        make: || plain_row(Presence::Present, Emphasis::Plain),
    },
    Case {
        component: "list_row",
        state: "selected",
        make: || {
            row(
                Presence::Present,
                Emphasis::Plain,
                Selection::Selected,
                Check::Off,
            )
        },
    },
    Case {
        component: "list_row",
        state: "starred-at-rest",
        make: || {
            row(
                Presence::Present,
                Emphasis::Plain,
                Selection::Unselected,
                Check::On,
            )
        },
    },
    Case {
        component: "list_row",
        state: "entering",
        make: || plain_row(Presence::Entering, Emphasis::Strong),
    },
    Case {
        component: "list_row",
        state: "leaving-row",
        make: || plain_row(Presence::Leaving(Exit::Row), Emphasis::Plain),
    },
    Case {
        component: "list_row",
        state: "leaving-row-unread",
        make: || plain_row(Presence::Leaving(Exit::Row), Emphasis::Strong),
    },
    Case {
        component: "list_row",
        state: "drag-source",
        make: || dragged_row(DropState::Source),
    },
    Case {
        component: "list_row",
        state: "drop-target",
        make: || dragged_row(DropState::Target),
    },
    Case {
        component: "list_row",
        state: "healing",
        make: healing_row,
    },
    Case {
        component: "list_row",
        state: "bare",
        make: || {
            rsx! {
                ListRow {
                    state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                    presence: Presence::Present,
                    name: "Sam Lindqvist",
                    via: None,
                    subject: "Notes from the sync review",
                    snippet: None,
                    time: "Tue",
                    tags: rsx! {},
                    star: None,
                    strip: None,
                    onclick: |_| {},
                }
            }
        },
    },
    // Gallery fix A: a name longer than the column's 26-character budget fades; "bare" above
    // is the one that fits and does not.
    Case {
        component: "list_row",
        state: "name-overflowing",
        make: || {
            rsx! {
                ListRow {
                    state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                    presence: Presence::Present,
                    name: "Maximilian Alexander von Hohenberg-Wittelsbach",
                    via: None,
                    subject: "Notes from the sync review",
                    snippet: None,
                    time: "Tue",
                    tags: rsx! {},
                    star: None,
                    strip: None,
                    onclick: |_| {},
                }
            }
        },
    },
    // AnimatedList.
    Case {
        component: "animated_list",
        state: "entering",
        make: || rsx! { AnimatedList { label: "Threads", {plain_row(Presence::Entering, Emphasis::Strong)} } },
    },
    Case {
        component: "animated_list",
        state: "present",
        make: || rsx! { AnimatedList { label: "Threads", {plain_row(Presence::Present, Emphasis::Plain)} } },
    },
];
