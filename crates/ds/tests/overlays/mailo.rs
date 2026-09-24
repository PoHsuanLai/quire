//! The mailo gaps 2 overlay cases: the command panel's opaque entrance, palette rows whose title
//! and detail are runs with a trailing remove, and the same rows in a menu.

use crate::cases::Case;
use dioxus::prelude::*;
use ds::{
    CommandPalette, Icon, MenuEntry, MenuRow, PaletteEntrance, RowAction, Run, RunTone, Text, Tile,
};
use std::time::Duration;

const NOW: Duration = Duration::ZERO;

/// A remove button that does nothing.
pub fn remove() -> RowAction {
    RowAction {
        icon: Icon::X,
        label: "Remove from recent".to_string(),
        on_press: EventHandler::new(|_| {}),
    }
}

/// Recent searches: a name stronger than its path, a caller's mark, a plain title the query
/// marks, and a trailing remove on the first two.
pub fn recent_rows() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Row(MenuRow {
            detail: Some(Text::Runs(vec![
                Run::new("in ", RunTone::Faint),
                Run::new("Inbox", RunTone::Plain),
            ])),
            tile: Some(Tile::Icon(Icon::Clock)),
            trailing: Some(remove()),
            ..MenuRow::new(
                1,
                Text::Runs(vec![
                    Run::new("from:dana ", RunTone::Strong),
                    Run::new("uidl", RunTone::Mark),
                ]),
            )
        }),
        MenuEntry::Row(MenuRow {
            tile: Some(Tile::Icon(Icon::Clock)),
            trailing: Some(remove()),
            ..MenuRow::new(2, "invoice september")
        }),
        MenuRow::new(3, "Sync now").into(),
    ]
}

fn groups() -> Vec<(String, Vec<MenuEntry<u8>>)> {
    vec![("Recent".to_string(), recent_rows())]
}

pub const MAILO_CASES: &[Case] = &[
    Case {
        component: "command_palette",
        state: "opaque-entrance",
        make: || rsx! { CommandPalette { label: "Search and commands", placeholder: "Search mail, people, actions", query: "", tokens: Vec::new(), groups: groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {}, entrance: PaletteEntrance::Opaque } },
        wait: NOW,
    },
    Case {
        component: "command_palette",
        state: "runs-and-trailing",
        make: || rsx! { CommandPalette { label: "Search and commands", placeholder: "Search mail, people, actions", query: "in", tokens: Vec::new(), groups: groups(), empty: "Nothing matches.", oninput: |_| {}, onpick: |_: u8| {}, onclose: |_| {} } },
        wait: NOW,
    },
];
