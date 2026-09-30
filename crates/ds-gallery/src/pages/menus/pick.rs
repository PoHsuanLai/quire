//! Menu hints and toggles, and PickList (design/30 section 2.4): a trailing hint in place of a
//! second line, items that keep their menu open, and a filterable list in a popover.

use crate::axes::{Axes, Showcase};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::host::measure::{Anchor, MountedRef, use_rect};
use ds::prelude::*;

/// Snooze times with the time each lands on, as the Mac's menu would carry it: at the end.
fn snooze() -> Vec<MenuItem<u8>> {
    vec![
        MenuItem::Header("Snooze until".to_string()),
        MenuItem::new(1, "Later today").with_hint("6:00 PM"),
        MenuItem::new(2, "Tomorrow").with_hint("Thu 8:00 AM"),
        MenuItem::new(3, "This weekend").with_hint("Sat 9:00 AM"),
        MenuItem::new(4, "Next week").with_hint("Mon 8:00 AM"),
        MenuItem::Separator,
        MenuItem::new(5, "Pick a date…")
            .with_key(Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('d')])),
    ]
}

/// A set of labels that toggle in place: every label keeps the menu open.
fn label_menu(on: &[u8]) -> Vec<MenuItem<u8>> {
    let label = |value: u8, name: &str| {
        let state = if on.contains(&value) {
            Check::On
        } else {
            Check::Off
        };
        MenuItem::new(value, name)
            .with_check(state)
            .with_after(AfterPick::KeepOpen)
    };
    vec![
        MenuItem::Header("Labels".to_string()),
        label(1, "Invoices"),
        label(2, "Travel"),
        label(3, "Family"),
        MenuItem::Separator,
        MenuItem::new(9, "Done"),
    ]
}

/// The folders a thread can move to, narrowed by what was typed, or the labels.
const NAMES: [&str; 6] = [
    "Invoices",
    "Travel",
    "Family",
    "Receipts",
    "Trash bin",
    "Travel 2025",
];

/// The labels containing `query`, each a toggle, and a row that creates one from the query.
fn label_groups(query: &str, on: &[u8]) -> Vec<PaletteGroup<u8>> {
    let needle = query.to_lowercase();
    let mut rows: Vec<PaletteRow<u8>> = (1u8..)
        .zip(NAMES)
        .filter(|(_, name)| name.to_lowercase().contains(&needle))
        .map(|(value, name)| PaletteRow {
            accessory: Accessory::Check(if on.contains(&value) {
                Check::On
            } else {
                Check::Off
            }),
            after: AfterPick::KeepOpen,
            ..PaletteRow::new(value, name)
        })
        .collect();
    if !query.is_empty() {
        rows.push(PaletteRow {
            leading: RowLeading::Icon(Icon::Plus),
            ..PaletteRow::new(99, format!("Create “{query}”"))
        });
    }
    vec![PaletteGroup::list("Labels", rows)]
}

/// `value` in `on` if it was not, out of it if it was.
fn flip(mut on: Signal<Vec<u8>>, value: u8) {
    on.with_mut(|on| match on.iter().position(|at| *at == value) {
        Some(at) => {
            on.remove(at);
        }
        None => on.push(value),
    });
}

/// The hint specimen, the toggling menu and the filterable list.
#[component]
pub fn HintsAndPickList() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let on = use_signal(|| vec![1u8]);
    let mut button = use_signal(|| None::<MountedRef>);
    let mut pick = use_signal(|| showcase == Showcase::Posed);
    let mut query = use_signal(|| match showcase {
        Showcase::Posed => "tra".to_string(),
        Showcase::Live => String::new(),
    });
    let opener = use_rect();
    let anchor = match (showcase, opener.rect()) {
        (Showcase::Posed, Some(rect)) => Some(Anchor::Rect(rect)),
        (Showcase::Posed, None) => None,
        (Showcase::Live, _) => button().map(Anchor::Mounted),
    };
    rsx! {
        Section {
            title: "Menu: hints and toggles",
            note: "The Mac's menu has no second line, so what an item adds to its title (the time a snooze lands on) stands at its end, faint, before any key equivalent. An item with AfterPick::KeepOpen yields at once and leaves the menu up with no blink, so a set of toggles is one menu; the caller redraws the checks.",
            div { class: "g-row g-row-top",
                Specimen { name: "Hints", code: "MenuItem::new(1, \"Later today\").with_hint(\"6:00 PM\")".to_string(),
                    div { class: "g-menu-card",
                        Menu::<u8> {
                            placement: MenuPlacement::Popup,
                            anchor: Anchor::Point(Point::default()),
                            items: snooze(),
                            flow: Flow::Inline,
                            active: MenuCursor::Controlled(Some(1)),
                            onpick: |_| {},
                            onclose: |_| {},
                        }
                    }
                }
                Specimen { name: "Keeps open", code: "MenuItem::new(1, \"Invoices\").with_after(AfterPick::KeepOpen)".to_string(),
                    div { class: "g-menu-card",
                        Menu::<u8> {
                            placement: MenuPlacement::Popup,
                            anchor: Anchor::Point(Point::default()),
                            items: label_menu(&on()),
                            flow: Flow::Inline,
                            onpick: move |value: u8| {
                                if value < 9 {
                                    flip(on, value);
                                }
                            },
                            onclose: |_| {},
                        }
                    }
                }
            }
        }
        Section {
            title: "PickList",
            note: "A search field over rows in a popover under its control. The caller narrows the rows by the query it hears; a row that keeps the list open toggles in place, a Create row or a Move to row closes it. Up and Down move, Return picks, Escape closes.",
            div { class: "g-row",
                div {
                    onmounted: move |event| {
                        opener.on_mounted(event.clone());
                        button.set(Some(MountedRef(event.data())));
                    },
                    Button {
                        label: "Labels…",
                        value: Some(if pick() { Check::On } else { Check::Off }),
                        onclick: move |_| pick.set(!pick()),
                    }
                }
            }
            div { style: "height:300px" }
            if let (true, Some(anchor)) = (pick(), anchor) {
                PickList::<u8> {
                    anchor,
                    label: "Labels",
                    placeholder: "Filter labels",
                    query: query(),
                    groups: label_groups(&query(), &on()),
                    empty: "No label matches.",
                    oninput: move |text: String| query.set(text),
                    onpick: move |value: u8| {
                        if value == 99 {
                            query.set(String::new());
                        } else {
                            flip(on, value);
                        }
                    },
                    onclose: move |()| pick.set(false),
                }
            }
        }
    }
}
