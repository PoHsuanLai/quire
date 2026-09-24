//! Overlays, mailo gaps 2: a command panel that is opaque from its first frame, rows whose title
//! and detail are the caller's runs with a trailing remove.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    CommandPalette, CommandPaletteHost, Corner, Icon, Material, MenuEntry, MenuRow,
    PaletteEntrance, Radius, RowAction, Run, RunTone, Surface, Text, Tile,
};

/// The recent searches a panel starts with.
const RECENT: [(&str, &str); 3] = [
    ("from:dana", "uidl"),
    ("label:invoices", "september"),
    ("in:sent", "photos"),
];

/// A recent search as a row: the operator strong, the words marked, a remove at its end.
fn recent_row(index: usize, mut kept: Signal<Vec<usize>>) -> MenuEntry<u8> {
    let (operator, words) = RECENT[index];
    MenuEntry::Row(MenuRow {
        detail: Some(Text::Runs(vec![
            Run::new("searched ", RunTone::Faint),
            Run::new("today", RunTone::Plain),
        ])),
        tile: Some(Tile::Icon(Icon::Clock)),
        trailing: Some(RowAction {
            icon: Icon::X,
            label: "Remove from recent".to_string(),
            on_press: EventHandler::new(move |_| {
                kept.with_mut(|kept| kept.retain(|at| *at != index))
            }),
        }),
        ..MenuRow::new(
            u8::try_from(index).unwrap_or_default(),
            Text::Runs(vec![
                Run::new(format!("{operator} "), RunTone::Strong),
                Run::new(words, RunTone::Mark),
            ]),
        )
    })
}

/// Recent searches in a panel: the × removes one without running it.
#[component]
pub fn RecentPalette() -> Element {
    let kept = use_signal(|| vec![0usize, 1, 2]);
    let rows: Vec<MenuEntry<u8>> = kept()
        .into_iter()
        .map(|index| recent_row(index, kept))
        .collect();
    rsx! {
        Section {
            title: "Command panel: opaque entrance, runs, trailing actions",
            note: "PaletteEntrance::Opaque springs with cmdk-rise, which has no fade: the first frame is already opaque. A MenuRow's title and detail are Text runs (the operator strong, the words marked); its trailing RowAction removes the search without running it or moving the selection.",
            div { class: "g-row g-row-top",
                Specimen { name: "Recent searches", code: "MenuEntry::Row(MenuRow { trailing: Some(RowAction { .. }), .. })".to_string(),
                    div { class: "g-launcher",
                        Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                            CommandPalette::<u8> {
                                label: "Search and commands",
                                placeholder: "Search mail, people, actions",
                                query: String::new(),
                                tokens: Vec::new(),
                                groups: vec![("Recent".to_string(), rows)],
                                empty: "No recent searches.",
                                oninput: |_| {},
                                onpick: |_| {},
                                onclose: |_| {},
                                host: CommandPaletteHost::Surface,
                                entrance: PaletteEntrance::Opaque,
                            }
                        }
                    }
                }
            }
        }
    }
}
