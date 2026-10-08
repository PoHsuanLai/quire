//! A control in a formatting bar never takes the keyboard from the text body (mailo's format
//! bubble over a selection). Pressing a `SegmentedControl` segment in a bubble over an
//! `EditSurface` still acts, but the surface keeps the keyboard and the caret: the surface hears
//! no blur, reports no new pointer position, and a later command-I reaches its key handler.

use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::edit::handle::{EditHandle, use_edit_handle};
use ds::edit::input::{EditInput, KeyInput};
use ds::edit::pointer::{EditFocus, EditPointer};
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    static INPUT: RefCell<Vec<EditInput>> = const { RefCell::new(Vec::new()) };
    static POINTER: RefCell<Vec<EditPointer>> = const { RefCell::new(Vec::new()) };
    static FOCUS: RefCell<Vec<EditFocus>> = const { RefCell::new(Vec::new()) };
}

fn drain<T>(log: &'static std::thread::LocalKey<RefCell<Vec<T>>>) -> Vec<T> {
    log.with(|log| log.borrow_mut().drain(..).collect())
}

#[derive(Debug, Clone, PartialEq)]
enum Style {
    Plain,
    Bold,
}

/// A surface with a format bubble drawn over its text (a sibling, absolutely positioned).
#[allow(non_snake_case)]
fn Editor() -> Element {
    let handle: EditHandle = use_edit_handle();
    let mut style = use_signal(|| Style::Plain);
    let mut picked = use_signal(|| 0u32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "position:relative; padding:20px; width:300px; font-size:16px; line-height:20px",
                EditSurface {
                    common: Common { id: Some("editor".to_string()), ..Common::default() },
                    handle,
                    on_input: |input| INPUT.with(|log| log.borrow_mut().push(input)),
                    on_pointer: |pointer| POINTER.with(|log| log.borrow_mut().push(pointer)),
                    on_focus: |focus| FOCUS.with(|log| log.borrow_mut().push(focus)),
                    p { id: "one", "data-edit-node": "p0", style: "margin:0", "Hello world" }
                }
                div { id: "bubble", style: "position:absolute; left:20px; top:50px",
                    onmousedown: move |event: MouseEvent| event.prevent_default(),
                    FocusOnPressScope { focus: FocusOnPress::Refuses,
                    SegmentedControl {
                        label: "Style",
                        choices: vec![
                            Choice::new(Style::Plain, "Plain"),
                            Choice::new(Style::Bold, "Bold"),
                        ],
                        tracking: Tracking::SelectOne(style()),
                        onchange: move |value| {
                            picked += 1;
                            style.set(value);
                        },
                    }
                    }
                }
                p { class: "log", "{picked}" }
            }
        }
    }
}

fn fresh(fallback: FocusFallback) -> Harness {
    drain(&INPUT);
    drain(&POINTER);
    drain(&FOCUS);
    let mut harness = Harness::new(
        Editor,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_focus_fallback(fallback),
    );
    harness.advance(Duration::from_millis(50));
    harness
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} missing:\n{}", harness.html()))
}

fn bubble_press_keeps_the_surface(fallback: FocusFallback) {
    let mut harness = fresh(fallback);
    let one = harness.rect("#one").expect("the paragraph");
    let from = Point {
        x: Px(one.origin.x.0 + 1.0),
        y: Px(one.origin.y.0 + 10.0),
    };
    let to = Point {
        x: Px(one.origin.x.0 + 40.0),
        y: Px(one.origin.y.0 + 10.0),
    };
    harness.send(Input::drag(from, to, 3));
    harness.advance(Duration::from_millis(600));
    assert_eq!(harness.focus_of("#editor"), FocusState::Focused);
    drain(&POINTER);
    drain(&FOCUS);
    drain(&INPUT);

    harness.send(Input::click(centre(
        &harness,
        ".ds-segmented-segment:last-of-type",
    )));
    harness.advance(Duration::from_millis(200));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("1"),
        "the press still acted"
    );
    assert_eq!(
        harness.focus_of("#editor"),
        FocusState::Focused,
        "the body kept the keyboard"
    );
    assert_eq!(drain(&FOCUS), Vec::new(), "the surface heard no blur");
    assert_eq!(drain(&POINTER), Vec::new(), "the selection did not move");

    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Char('i')));
    let keys: Vec<KeyInput> = drain(&INPUT)
        .into_iter()
        .filter_map(|input| match input {
            EditInput::Key(key) => Some(key),
            _ => None,
        })
        .collect();
    assert!(
        keys.iter().any(
            |key| key.modifiers.contains(keyboard_types::Modifiers::SUPER)
                && key.key == Key::Character("i".into())
        ),
        "command-I reached the edit's key handler: {keys:?}"
    );
}

#[test]
fn a_format_bar_segment_press_keeps_the_keyboard_in_the_surface() {
    bubble_press_keeps_the_surface(FocusFallback::Ancestor);
}

#[test]
fn a_format_bar_segment_press_keeps_the_keyboard_under_blitz_default() {
    bubble_press_keeps_the_surface(FocusFallback::BlitzDefault);
}
