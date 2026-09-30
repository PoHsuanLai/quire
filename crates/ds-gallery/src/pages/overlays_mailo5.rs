//! Overlays, for mail: a menu whose filter draws the typed query in a field row.

use super::Section;
use dioxus::prelude::*;
use ds::{Button, Flow, Menu, MenuEntry, MenuFilter, MenuKind, MenuRow, Trailing, use_rect};

/// The folders a thread can move to.
const FOLDERS: [&str; 6] = [
    "Invoices",
    "Travel",
    "Family",
    "Receipts",
    "Archive/2025",
    "Archive/2026",
];

/// The folder rows.
fn folders() -> Vec<MenuEntry<usize>> {
    FOLDERS
        .into_iter()
        .enumerate()
        .map(|(value, name)| MenuEntry::Row(MenuRow::new(value, name)))
        .collect()
}

/// The field filter the pickers use.
fn field() -> MenuFilter {
    MenuFilter::Field {
        placeholder: "Filter folders…".to_string(),
    }
}

/// A folder picker whose typed query is drawn: floating from a trigger, and inline in a card.
#[component]
pub fn FieldFilterMenu() -> Element {
    let mut open = use_signal(|| false);
    let mut moved = use_signal(|| "nowhere yet".to_string());
    let trigger = use_rect();
    rsx! {
        Section {
            title: "A menu that shows its filter",
            note: "filter: Filter::Field {{ placeholder }} types and filters as Filter::Typing does, and draws the query in a field row at the top; the cursor stays on the rows. Open the picker and type.",
            div { class: "g-row g-row-top",
                div { onmounted: move |event| trigger.on_mounted(event),
                    Button { label: "Move to…", trailing: Trailing::Caret, onclick: move |_| open.set(true) }
                }
                p { class: "g-note", "Moved to: {moved}" }
                div { class: "g-menu-card",
                    Menu::<usize> {
                        kind: MenuKind::Rich,
                        anchor: ds::Anchor::Point(ds::Point::default()),
                        entries: folders(),
                        filter: field(),
                        flow: Flow::Inline,
                        onpick: move |value: usize| moved.set(FOLDERS[value].to_string()),
                        onclose: |_| {},
                    }
                }
            }
            if let (true, Some(anchor)) = (open(), trigger.anchor()) {
                Menu::<usize> {
                    kind: MenuKind::Dropdown,
                    anchor,
                    entries: folders(),
                    filter: field(),
                    onpick: move |value: usize| moved.set(FOLDERS[value].to_string()),
                    onclose: move |()| open.set(false),
                }
            }
        }
    }
}
