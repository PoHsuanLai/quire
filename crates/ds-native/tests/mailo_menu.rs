//! mailo gaps 2, menus, on a real Blitz document: a field beside a menu drives its highlight
//! and keeps the keyboard, the pointer only asks; a typed filter is heard as it changes; a row's
//! trailing action fires without picking it or closing the menu.

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Cursor, Ds, Filter, Focus, Icon, InputVariant, Key, Material, Menu,
    MenuEntry, MenuKind, MenuRow, Point, Px, RowAction, TextInput,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const NAMES: [&str; 4] = ["Dana Okafor", "Sam Lindqvist", "Priya Raman", "Mei Chen"];

/// The people menu's rows, each with a remove that logs it.
fn people(mut log: Signal<Vec<String>>) -> Vec<MenuEntry<u8>> {
    (0u8..)
        .zip(NAMES)
        .map(|(value, name)| {
            MenuEntry::Row(MenuRow {
                trailing: Some(RowAction {
                    icon: Icon::X,
                    label: format!("Forget {name}"),
                    on_press: EventHandler::new(move |_| {
                        log.with_mut(|log| log.push(format!("remove:{value}")))
                    }),
                }),
                ..MenuRow::new(value, name)
            })
        })
        .collect()
}

/// Whose cursor the page's menu shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drive {
    /// The field's: Up and Down in the field move it; the pointer's requests are only logged.
    Field,
    /// The menu's own, with its typed filter.
    Own,
}

#[component]
fn Page(drive: Drive) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut at = use_signal(|| 0usize);
    let mut open = use_signal(|| true);
    let active = match drive {
        Drive::Field => Cursor::Controlled(Some(at())),
        Drive::Own => Cursor::Auto,
    };
    let filter = match drive {
        Drive::Field => Filter::None,
        Drive::Own => Filter::Typing,
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:460px; padding:20px",
                div { class: "field",
                    TextInput {
                        variant: InputVariant::Boxed,
                        label: "To",
                        value: String::new(),
                        oninput: move |_| {},
                        focus: Focus::OnMount,
                        onkey: move |event: KeyboardEvent| match event.key() {
                            dioxus::prelude::Key::ArrowDown => at.set((at() + 1).min(NAMES.len() - 1)),
                            dioxus::prelude::Key::ArrowUp => at.set(at().saturating_sub(1)),
                            _ => {}
                        },
                    }
                }
                p { class: "log", {log().join(",")} }
            }
            if open() {
                Menu::<u8> {
                    kind: MenuKind::Rich,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(120.0) }),
                    entries: people(log),
                    filter,
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onclose: move |()| open.set(false),
                    active,
                    on_active: move |index: Option<usize>| log.with_mut(|log| log.push(format!("active:{index:?}"))),
                    onquery: move |text: String| log.with_mut(|log| log.push(format!("query:{text}"))),
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn FieldDriven() -> Element {
    rsx! { Page { drive: Drive::Field } }
}

#[allow(non_snake_case)]
fn OwnCursor() -> Element {
    rsx! { Page { drive: Drive::Own } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn highlighted(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-menu-item[*|aria-selected=true] .ds-menu-title")
}

#[test]
fn a_field_drives_the_highlight_and_keeps_the_keyboard() {
    let mut harness = Harness::new(FieldDriven, VIEW);
    harness.advance(ms(300));
    assert_eq!(highlighted(&harness).as_deref(), Some("Dana Okafor"));
    assert!(
        harness.is_focused(".field .ds-input"),
        "the field kept the keyboard"
    );
    harness.key(Key::Down);
    harness.key(Key::Down);
    harness.advance(ms(50));
    assert_eq!(highlighted(&harness).as_deref(), Some("Priya Raman"));
    assert!(harness.is_focused(".field .ds-input"));
    harness.key(Key::Up);
    harness.advance(ms(50));
    assert_eq!(highlighted(&harness).as_deref(), Some("Sam Lindqvist"));
    // The pointer over the last row asks for it; the page does not follow, so it stays.
    let last = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-title")
        .expect("the last row");
    harness.pointer_move(last);
    harness.advance(ms(50));
    assert_eq!(log(&harness), "active:Some(3)");
    assert_eq!(highlighted(&harness).as_deref(), Some("Sam Lindqvist"));
}

#[test]
fn the_typed_filter_is_heard_and_the_own_cursor_reported() {
    let mut harness = Harness::new(OwnCursor, VIEW);
    harness.advance(ms(300));
    assert!(
        harness.is_focused(".ds-menu"),
        "an own cursor takes the keyboard"
    );
    harness.key(Key::Down);
    harness.advance(ms(50));
    harness.key(Key::Char('m'));
    harness.key(Key::Char('e'));
    harness.key(Key::Backspace);
    harness.advance(ms(50));
    assert_eq!(
        log(&harness),
        "active:Some(0),active:Some(1),query:m,active:Some(0),query:me,query:m"
    );
}

#[test]
fn a_trailing_action_in_a_menu_neither_picks_nor_closes() {
    let mut harness = Harness::new(OwnCursor, VIEW);
    harness.advance(ms(300));
    let remove = harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-action .ds-icon-button")
        .expect("the second row's remove");
    harness.click(remove);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "active:Some(0),remove:1");
    assert_eq!(harness.count(".ds-menu"), 1, "the menu is still open");
}
