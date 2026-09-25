//! The mailo gaps 5 overlay cases: a scrim drawn inline in the pane it dims, under the reader
//! the pane draws after it; a menu whose filter draws its query line.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{Anchor, Filter, Flow, Menu, MenuEntry, MenuKind, MenuRow, Point, Px, Scrim};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;

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
