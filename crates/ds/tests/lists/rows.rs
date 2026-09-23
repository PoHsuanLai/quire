//! The row cases: ListRow in every weight, star and motion state, and AnimatedList, with the
//! helpers that draw a thread row the way a consumer does.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::components::vocab::{Emphasis, PulseKey, Selection, StaggerIndex, Switch};
use ds::{
    ActionId, Anim, AnimatedList, Chip, ChipVariant, Exit, HoverStrip, Icon, ListPresence, ListRow,
    MarkSize, MarkStyle, Presence, Provider, ProviderMark, Px, StripAction,
};

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
pub fn row(
    presence: Presence,
    emphasis: Emphasis,
    selection: Selection,
    star: Switch,
    pulse: PulseKey,
    index: usize,
) -> Element {
    rsx! {
        ListRow {
            selection,
            emphasis,
            index: StaggerIndex::new(index),
            presence,
            name: "Dana Okafor",
            via: rsx! {
                ProviderMark { provider: Provider::Google, size: MarkSize::Row, style: MarkStyle::Letter }
                "gmail"
            },
            subject: "Re: UIDL stability across servers",
            snippet: "Treat UIDL as stable only while UIDVALIDITY holds.".to_string(),
            time: "09:41",
            tags: rsx! { Chip { variant: ChipVariant::Accent, text: "spec" } },
            star: (star, EventHandler::new(|_| {})),
            star_pulse: pulse,
            strip: rsx! { HoverStrip { actions: strip_actions() } },
            onclick: |_| {},
        }
    }
}

/// A read, unselected, unstarred row: what a roster draws per entry, keyed by the consumer.
#[component]
pub fn Row(presence: Presence, emphasis: Emphasis, index: StaggerIndex) -> Element {
    row(
        presence,
        emphasis,
        Selection::Unselected,
        Switch::Off,
        PulseKey::rest(Anim::StarPop),
        usize::from(index.get()),
    )
}

fn plain_row(presence: Presence, emphasis: Emphasis) -> Element {
    row(
        presence,
        emphasis,
        Selection::Unselected,
        Switch::Off,
        PulseKey::rest(Anim::StarPop),
        3,
    )
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
                Switch::Off,
                PulseKey::rest(Anim::StarPop),
                0,
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
                Switch::On,
                PulseKey::rest(Anim::StarPop),
                0,
            )
        },
    },
    Case {
        component: "list_row",
        state: "starring-pop-a",
        make: || {
            row(
                Presence::Present,
                Emphasis::Plain,
                Selection::Unselected,
                Switch::On,
                PulseKey::rest(Anim::StarPop).fired(),
                0,
            )
        },
    },
    Case {
        component: "list_row",
        state: "starring-pop-b",
        make: || {
            row(
                Presence::Present,
                Emphasis::Plain,
                Selection::Unselected,
                Switch::On,
                PulseKey::rest(Anim::StarPop).fired().fired(),
                0,
            )
        },
    },
    Case {
        component: "list_row",
        state: "unstarring-pop",
        make: || {
            row(
                Presence::Present,
                Emphasis::Plain,
                Selection::Unselected,
                Switch::Off,
                PulseKey::rest(Anim::StarPop).fired(),
                0,
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
        state: "leaving-fold",
        make: || plain_row(Presence::Leaving(Exit::Fold), Emphasis::Plain),
    },
    Case {
        component: "list_row",
        state: "leaving-fold-unread",
        make: || plain_row(Presence::Leaving(Exit::Fold), Emphasis::Strong),
    },
    Case {
        component: "list_row",
        state: "leaving-curl",
        make: || plain_row(Presence::Leaving(Exit::Curl), Emphasis::Plain),
    },
    Case {
        component: "list_row",
        state: "leaving-crumple",
        make: || plain_row(Presence::Leaving(Exit::Crumple), Emphasis::Plain),
    },
    Case {
        component: "list_row",
        state: "healing",
        make: || {
            plain_row(
                Presence::Healing {
                    dy: Px(79.0),
                    d: StaggerIndex::new(1),
                },
                Emphasis::Plain,
            )
        },
    },
    Case {
        component: "list_row",
        state: "bare",
        make: || {
            rsx! {
                ListRow {
                    selection: Selection::Unselected,
                    emphasis: Emphasis::Plain,
                    index: StaggerIndex::new(0),
                    presence: Presence::Present,
                    name: "Sam Lindqvist",
                    via: None,
                    subject: "Notes from the sync review",
                    snippet: None,
                    time: "Tue",
                    tags: rsx! {},
                    star: None,
                    star_pulse: PulseKey::rest(Anim::StarPop),
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
        make: || rsx! { AnimatedList { label: "Threads", presence: ListPresence::Entering, {plain_row(Presence::Entering, Emphasis::Strong)} } },
    },
    Case {
        component: "animated_list",
        state: "present",
        make: || rsx! { AnimatedList { label: "Threads", presence: ListPresence::Present, {plain_row(Presence::Present, Emphasis::Plain)} } },
    },
];
