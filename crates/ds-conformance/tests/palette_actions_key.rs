//! The actions key toggling an actions menu over the palette (design/06 §20.2):
//! the first Ctrl+K reaches the palette's `onkey` and opens the menu; the second arrives
//! while the menu has the keyboard, so the caller reads it where the menu's keydown bubbles, at
//! its own root, as CONSUMING.md "palette follow-ups" says.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::PaletteGroup;
use ds::components::menus::palette::palette_group::PaletteRow;
use ds::focus::request::use_focus_request;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Driver, FocusState, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str) -> PaletteRow<u8> {
    PaletteRow::new(value, title.to_string())
}

/// Whether `event` is the actions key, Ctrl+K.
fn is_actions(event: &KeyboardEvent) -> bool {
    event.modifiers().ctrl() && event.key() == dioxus::prelude::Key::Character("k".to_string())
}

#[allow(non_snake_case)]
fn Launcher() -> Element {
    let mut actions = use_signal(|| false);
    let mut row = use_signal(|| None::<Rect>);
    let field = use_focus_request();
    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if actions() && is_actions(&event) {
                    event.prevent_default();
                    actions.set(false);
                    field.request();
                }
            },
            Ds { appearance: Appearance::default(), material: Material::Sheet,
                div { style: "width:600px; height:400px",
                    CommandPalette::<u8> {
                        label: "Launch".to_string(),
                        placeholder: "Search".to_string(),
                        query: String::new(),
                        tokens: Vec::new(),
                        groups: vec![PaletteGroup::list("Applications", vec![item(1, "Files"), item(2, "Firefox")])],
                        empty: "Nothing".to_string(),
                        oninput: move |_| {},
                        onpick: move |_| {},
                        onclose: move |()| {},
                        host: CommandPaletteHost::Surface,
                        id: "card".to_string(),
                        focus: field,
                        on_select_rect: move |rect: Rect| row.set(Some(rect)),
                        onkey: move |event: KeyboardEvent| {
                            if is_actions(&event) {
                                // Stopped here too: bubbling on, it would reach the root's
                                // handler with the menu already open and close it again.
                                event.prevent_default();
                                event.stop_propagation();
                                actions.set(true);
                            }
                        },
                    }
                }
                if let (true, Some(rect)) = (actions(), row()) {
                    Menu::<u8> {
                        placement: MenuPlacement::Popup,
                        anchor: Anchor::Rect(rect),
                        items: vec![MenuItem::new(9, "Open"), MenuItem::new(10, "Quit")],
                        onpick: move |_| {},
                        onclose: move |()| {
                            actions.set(false);
                            field.request();
                        },
                    }
                }
            }
        }
    }
}

#[test]
fn a_second_actions_key_closes_the_menu_and_gives_the_field_the_keyboard_back() {
    let mut harness = Harness::new(Launcher, VIEW);
    harness.advance(ms(300));
    assert_eq!(harness.focus_of("#card .ds-input"), FocusState::Focused);
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('k')));
    harness.advance(ms(200));
    assert_eq!(
        harness.count(".ds-popover.ds-menu"),
        1,
        "the first Ctrl+K opens it"
    );
    assert_eq!(
        harness.focus_of(".ds-popover.ds-menu"),
        FocusState::Focused,
        "the menu has the keyboard"
    );
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('k')));
    harness.advance(ms(200));
    assert_eq!(
        harness.count(".ds-popover.ds-menu"),
        0,
        "the second Ctrl+K closes it"
    );
    assert_eq!(
        harness.focus_of("#card .ds-input"),
        FocusState::Focused,
        "the field has the keyboard back"
    );
}
