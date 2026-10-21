//! A sheet opened from a menu pick takes the keyboard, as a Mac sheet becomes the key window: the
//! menu that had the keyboard is gone, so without it no key reached the app (files' Choose
//! Application, opened from its context menu). Escape then closes the sheet.

use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonFocus;
use ds::focus::soon::focus_soon;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, FocusState, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 420,
    scale_percent: 100,
};

/// An app box that hears keys, a context menu whose one item opens a sheet with no first stop
/// of its own, and a log of what was heard.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut menu = use_signal(|| true);
    let mut sheet = use_signal(|| false);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                class: "app",
                tabindex: "0",
                style: "height:400px",
                onmounted: move |event: MountedEvent| focus_soon(event.data()),
                onkeydown: move |event: KeyboardEvent| {
                    log.with_mut(|log| log.push(format!("app:{}", event.key())));
                },
                p { class: "log", {log().join(",")} }
                if menu() {
                    Menu::<u8> {
                        placement: MenuPlacement::Context,
                        anchor: Anchor::Point(Point { x: Px(80.0), y: Px(60.0) }),
                        items: vec![MenuItem::new(0, "Open With…")],
                        onpick: move |_| sheet.set(true),
                        onclose: move |()| menu.set(false),
                    }
                }
                if sheet() {
                    Sheet {
                        label: "Choose Application".to_owned(),
                        onclose: move |()| {
                            sheet.set(false);
                            log.with_mut(|log| log.push("closed".to_string()));
                        },
                        p { "Applications" }
                        Button { label: "Cancel", onclick: move |_| sheet.set(false) }
                    }
                }
            }
        }
    }
}

#[test]
fn a_sheet_opened_from_a_menu_pick_takes_the_keyboard_and_escape_closes_it() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    let item = harness
        .centre(".ds-menu-item .ds-menu-label")
        .unwrap_or_else(|| panic!("no menu:\n{}", harness.html()));
    harness.send(Input::click(item));
    settle_until(&mut harness, |h| {
        h.count(".ds-sheet") == 1 && h.count(".ds-menu") == 0
    });
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.focus_of(".ds-sheet"),
        FocusState::Focused,
        "the sheet has the keyboard:\n{}",
        harness.html()
    );
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(Duration::from_millis(600));
    assert!(
        harness
            .text_of(".log")
            .unwrap_or_default()
            .contains("closed"),
        "Escape reached the sheet: {:?}",
        harness.text_of(".log")
    );
}

/// A sheet whose field, or else whose Cancel button, asks for the keyboard on mount.
#[component]
fn FirstStop(field: bool) -> Element {
    let field_focus = if field {
        FieldFocus::OnMount
    } else {
        FieldFocus::default()
    };
    let button_focus = if field {
        ButtonFocus::default()
    } else {
        ButtonFocus::OnMount
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:400px",
                Sheet { label: "Go to Folder".to_owned(), onclose: |()| {},
                    div { id: "path", style: "display:flex",
                        TextField { label: "Path", value: "", focus: field_focus, oninput: |_| {} }
                    }
                    div { id: "cancel",
                        Button { label: "Cancel", focus: button_focus, onclick: |_| {} }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn FieldFirst() -> Element {
    rsx! { FirstStop { field: true } }
}

#[allow(non_snake_case)]
fn ButtonFirst() -> Element {
    rsx! { FirstStop { field: false } }
}

#[test]
fn a_first_stop_the_caller_asks_for_still_takes_the_keyboard_from_the_sheet() {
    for (app, stop) in [
        (FieldFirst as fn() -> Element, "#path input"),
        (ButtonFirst, "#cancel button"),
    ] {
        let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        harness.advance(Duration::from_millis(600));
        assert_eq!(harness.focus_of(stop), FocusState::Focused, "{stop}");
        assert_eq!(
            harness.focus_of(".ds-sheet"),
            FocusState::Unfocused,
            "{stop}"
        );
    }
}
