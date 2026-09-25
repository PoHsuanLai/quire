//! The mailo gaps 5 overlay cases: a scrim drawn inline in the pane it dims, under the reader
//! the pane draws after it; a menu whose filter draws its query line; a hover card's flag whose
//! words are runs.

use crate::cases::{Case, PartsCard};
use dioxus::prelude::*;
use ds::{
    Anchor, Filter, FlagTone, Flow, HoverCardPart, Icon, Menu, MenuEntry, MenuKind, MenuRow, Point,
    Px, Run, RunTone, Scrim, Text,
};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;
/// Past the 450 ms hover intent.
const INTENT: Duration = Duration::from_millis(520);

/// The spoof warning: the brand and the domain in the strong tone.
fn spoof(tone: FlagTone) -> HoverCardPart {
    HoverCardPart::flag(
        tone,
        Icon::OctagonAlert,
        Text::Runs(vec![
            Run::new("Not ", RunTone::Plain),
            Run::new("Acme", RunTone::Strong),
            Run::new(": this was sent from ", RunTone::Plain),
            Run::new("acme-billing.example", RunTone::Strong),
            Run::new(".", RunTone::Plain),
        ]),
    )
}

/// The labels a thread can carry.
fn labels() -> Vec<MenuEntry<u8>> {
    ["Invoices", "Travel", "Family"]
        .into_iter()
        .zip(0u8..)
        .map(|(name, value)| MenuEntry::Row(MenuRow::new(value, name)))
        .collect()
}

/// A label picker whose filter is a drawn field.
fn field() -> Filter {
    Filter::Field {
        placeholder: "Filter labels…".to_string(),
    }
}

pub const MAILO5_CASES: &[Case] = &[
    Case {
        component: "hover_card",
        state: "part-flag-runs-danger",
        make: || rsx! { PartsCard { parts: vec![spoof(FlagTone::Danger)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-runs-info",
        make: || rsx! { PartsCard { parts: vec![spoof(FlagTone::Info)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-text-plain",
        make: || rsx! { PartsCard { parts: vec![HoverCardPart::flag(FlagTone::Danger, Icon::OctagonAlert, "Not the address Dana usually writes from.")] } },
        wait: INTENT,
    },
    Case {
        component: "menu",
        state: "filter-field",
        make: || rsx! { Menu { kind: MenuKind::Dropdown, anchor: Anchor::Point(Point { x: Px(20.0), y: Px(20.0) }), entries: labels(), filter: field(), onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
    Case {
        component: "menu",
        state: "filter-field-inline",
        make: || rsx! { div { Menu { kind: MenuKind::Rich, anchor: Anchor::Point(Point::default()), entries: labels(), filter: field(), onpick: |_: u8| {}, onclose: |_| {}, flow: Flow::Inline } } },
        wait: NOW,
    },
    Case {
        component: "scrim",
        state: "inline",
        make: || {
            rsx! {
                div { class: "pane", style: "position:relative",
                    p { "The list." }
                    Scrim { label: "Close peek", onclose: |_| {}, flow: Flow::Inline }
                    article { "The peeked reader." }
                }
            }
        },
        wait: NOW,
    },
];
