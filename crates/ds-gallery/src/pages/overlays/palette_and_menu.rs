//! Overlays, for mail: a command panel that is opaque from its first frame, rows whose title
//! and detail are the caller's runs with a trailing remove, and a people menu whose highlight
//! the field beside it drives.

use crate::axes::{Axes, Showcase};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::text_runs::RunTone;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::host::measure::{Anchor, use_rect};
use ds::prelude::*;
use ds::style::tokens::shape::{Corner, Radius};

/// The recent searches a panel starts with.
const RECENT: [(&str, &str); 3] = [
    ("from:dana", "uidl"),
    ("label:invoices", "september"),
    ("in:sent", "photos"),
];

/// A recent search as a row: the operator strong, the words marked, a remove at its end.
fn recent_row(index: usize, mut kept: Signal<Vec<usize>>) -> PaletteRow<u8> {
    let (operator, words) = RECENT[index];
    PaletteRow {
        detail: Some(TextLine::Runs(vec![
            TextRun::new("searched ", RunTone::Faint),
            TextRun::new("today", RunTone::Plain),
        ])),
        leading: RowLeading::Icon(Icon::Clock),
        action: Some(RowAction {
            icon: Icon::X,
            label: "Remove from recent".to_string(),
            on_press: EventHandler::new(move |_| {
                kept.with_mut(|kept| kept.retain(|at| *at != index))
            }),
        }),
        ..PaletteRow::new(
            u8::try_from(index).unwrap_or_default(),
            TextLine::Runs(vec![
                TextRun::new(format!("{operator} "), RunTone::Strong),
                TextRun::new(words, RunTone::Mark),
            ]),
        )
    }
}

/// Recent searches in a panel: the × removes one without running it.
#[component]
pub fn RecentPalette() -> Element {
    let kept = use_signal(|| vec![0usize, 1, 2]);
    let rows: Vec<PaletteRow<u8>> = kept()
        .into_iter()
        .map(|index| recent_row(index, kept))
        .collect();
    rsx! {
        Section {
            title: "Command panel: runs, trailing actions",
            note: "A PaletteRow's title and detail are TextLine runs (the operator strong, the words marked); its RowAction removes the search without running it or moving the selection.",
            div { class: "g-row g-row-top",
                Specimen { name: "Recent searches", code: "PaletteRow { action: Some(RowAction { .. }), .. }".to_string(),
                    div { class: "g-launcher",
                        Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                            CommandPalette::<u8> {
                                label: "Search and commands",
                                placeholder: "Search mail, people, actions",
                                query: String::new(),
                                tokens: Vec::new(),
                                groups: vec![PaletteGroup::list("Recent", rows)],
                                empty: "No recent searches.",
                                oninput: |_| {},
                                onpick: |_| {},
                                onclose: |_| {},
                                host: CommandPaletteHost::Surface,
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
    // Posed, the menu opens at the field's measured rect (a snapshot is taken before an
    // element anchor is measured); live, it anchors to the element itself.
    let field = use_rect();
    let people = use_signal(|| PEOPLE.to_vec());
    let rows: Vec<MenuItem<u8>> = (0u8..)
        .zip(people())
        .map(|(value, name)| MenuItem::new(value, name))
        .collect();
    let last = rows.len().saturating_sub(1);
    let anchor = match showcase {
        Showcase::Posed => field.rect().map(Anchor::Rect),
        Showcase::Live => field.anchor(),
    };
    rsx! {
        Section {
            title: "Menu driven by a field",
            note: "Cursor::Controlled: the field keeps the keyboard and its Up and Down move the highlight; the pointer only asks, through on_active.",
            div { class: "g-row",
                div { onmounted: move |event| field.on_mounted(event),
                    TextField {
                        label: "To",
                        value: String::new(),
                        placeholder: "Type a name, then Up and Down",
                        oninput: move |_| {},
                        focus: FieldFocus::Manual,
                        onkey: move |event: KeyboardEvent| match event.key() {
                            Key::ArrowDown => at.set((at() + 1).min(last)),
                            Key::ArrowUp => at.set(at().saturating_sub(1)),
                            _ => {}
                        },
                    }
                }
                Button { label: "Open the people menu", onclick: move |_| open.set(true) }
            }
            // Room under the field for the menu, inside the page.
            div { style: "height:230px" }
            if let (true, Some(anchor)) = (open(), anchor) {
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor,
                    items: rows,
                    onpick: move |_| open.set(false),
                    onclose: move |()| open.set(false),
                    active: MenuCursor::Controlled(Some(at())),
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
