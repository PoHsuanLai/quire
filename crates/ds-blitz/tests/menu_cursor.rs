//! Menus, on a real Blitz document: a field beside a menu drives its highlight and keeps the
//! keyboard, the pointer only asks; a menu's own cursor is heard as it moves.

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Ds, FieldFocus, Material, Menu, MenuCursor, MenuItem, MenuPlacement, Point,
    Px, ShortcutKey, TextField,
};
use ds_harness::{Harness, Viewport};
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

/// The people menu's items.
fn people() -> Vec<MenuItem<u8>> {
    (0u8..)
        .zip(NAMES)
        .map(|(value, name)| MenuItem::new(value, name))
        .collect()
}

/// Whose cursor the page's menu shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drive {
    /// The field's: Up and Down in the field move it; the pointer's requests are only logged.
    Field,
    /// The menu's own.
    Own,
}

#[component]
fn Page(drive: Drive) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut at = use_signal(|| 0usize);
    let mut open = use_signal(|| true);
    let active = match drive {
        Drive::Field => MenuCursor::Controlled(Some(at())),
        Drive::Own => MenuCursor::Auto,
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:460px; padding:20px",
                div { class: "field",
                    TextField {
                        label: "To",
                        value: String::new(),
                        oninput: move |_| {},
                        focus: FieldFocus::OnMount,
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
                    placement: MenuPlacement::Popup,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(120.0) }),
                    items: people(),
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onclose: move |()| open.set(false),
                    active,
                    on_active: move |index: Option<usize>| log.with_mut(|log| log.push(format!("active:{index:?}"))),
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
    harness.text_of(".ds-menu-item[*|data-selected=true] .ds-menu-label")
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
    harness.key(ShortcutKey::Down);
    harness.key(ShortcutKey::Down);
    harness.advance(ms(50));
    assert_eq!(highlighted(&harness).as_deref(), Some("Priya Raman"));
    assert!(harness.is_focused(".field .ds-input"));
    harness.key(ShortcutKey::Up);
    harness.advance(ms(50));
    assert_eq!(highlighted(&harness).as_deref(), Some("Sam Lindqvist"));
    // The pointer over the last row asks for it; the page does not follow, so it stays.
    let last = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-label")
        .expect("the last row");
    harness.pointer_move(last);
    harness.advance(ms(50));
    assert_eq!(log(&harness), "active:Some(3)");
    assert_eq!(highlighted(&harness).as_deref(), Some("Sam Lindqvist"));
}

#[test]
fn the_own_cursor_is_reported_as_it_moves() {
    let mut harness = Harness::new(OwnCursor, VIEW);
    harness.advance(ms(300));
    assert!(
        harness.is_focused(".ds-menu"),
        "an own cursor takes the keyboard"
    );
    harness.key(ShortcutKey::Down);
    harness.key(ShortcutKey::Down);
    harness.key(ShortcutKey::Up);
    harness.advance(ms(50));
    assert_eq!(
        log(&harness),
        "active:Some(0),active:Some(1),active:Some(2),active:Some(1)"
    );
}
