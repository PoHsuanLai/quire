//! Overlays, mailo gaps 2: a command panel that is opaque from its first frame, rows whose title
//! and detail are the caller's runs with a trailing remove, and a people menu whose highlight
//! the field beside it drives.

use super::{Section, Specimen};
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::{
    Anchor, Button, ButtonVariant, CommandPalette, CommandPaletteHost, Corner, Cursor, Focus, Icon,
    InputVariant, Material, Menu, MenuEntry, MenuKind, MenuRow, MountedRef, PaletteEntrance,
    Radius, RowAction, Run, RunTone, Surface, Text, TextInput, Tile,
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

/// The people a To field offers.
const PEOPLE: [&str; 4] = ["Dana Okafor", "Sam Lindqvist", "Priya Raman", "Mei Chen"];

/// A To field and the people menu under it: Up and Down in the field move the menu's
/// highlight (`Cursor::Controlled`), the field keeps the keyboard, and each person has a
/// forget button.
#[component]
pub fn FieldMenu() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut open = use_signal(|| showcase == Showcase::Posed);
    let mut at = use_signal(|| 1usize);
    let mut field = use_signal(|| None::<MountedRef>);
    let mut people = use_signal(|| PEOPLE.to_vec());
    let rows: Vec<MenuEntry<u8>> = (0u8..)
        .zip(people())
        .map(|(value, name)| {
            MenuEntry::Row(MenuRow {
                trailing: Some(RowAction {
                    icon: Icon::X,
                    label: format!("Forget {name}"),
                    on_press: EventHandler::new(move |_| {
                        people.with_mut(|people| people.retain(|seen| *seen != name))
                    }),
                }),
                ..MenuRow::new(value, name)
            })
        })
        .collect();
    let last = rows.len().saturating_sub(1);
    rsx! {
        Section {
            title: "Menu driven by a field",
            note: "Cursor::Controlled: the field keeps the keyboard and its Up and Down move the highlight; the pointer only asks, through on_active. Each person's forget button acts without picking.",
            div { class: "g-row",
                div { onmounted: move |event| field.set(Some(MountedRef(event.data()))),
                    TextInput {
                        variant: InputVariant::Boxed,
                        label: "To",
                        value: String::new(),
                        placeholder: "Type a name, then Up and Down",
                        oninput: move |_| {},
                        focus: Focus::Manual,
                        onkey: move |event: KeyboardEvent| match event.key() {
                            Key::ArrowDown => at.set((at() + 1).min(last)),
                            Key::ArrowUp => at.set(at().saturating_sub(1)),
                            _ => {}
                        },
                    }
                }
                Button { variant: ButtonVariant::Secondary, label: "Open the people menu", onclick: move |_| open.set(true) }
            }
            if let (true, Some(mounted)) = (open(), field()) {
                Menu::<u8> {
                    kind: MenuKind::Rich,
                    anchor: Anchor::Mounted(mounted),
                    entries: rows,
                    onpick: move |_| open.set(false),
                    onclose: move |()| open.set(false),
                    active: Cursor::Controlled(Some(at())),
                    on_active: move |index: Option<usize>| {
                        if let Some(index) = index {
                            at.set(index);
                        }
                    },
                }
            }
        }
    }
}
