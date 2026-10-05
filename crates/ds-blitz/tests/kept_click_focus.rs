//! A click a quire control keeps to itself still moves the keyboard (mailo, against v0.1.10).
//!
//! A strip button, a row's disclosure triangle and anything in its trailing slot stop their
//! click, so it never reached `Ds`'s click-focus fallback; Blitz then cleared the focus (or,
//! with the default prevented, left it where it was), and the keys went to `html`. Each control
//! now hands its kept click to the host itself: under `FocusFallback::Ancestor` the pressed
//! control (or its nearest focusable ancestor) has the keyboard afterwards, as a browser leaves
//! it. Under `FocusFallback::BlitzDefault` nothing changes: the negative control.

use dioxus::prelude::*;
use ds::base::press::Press;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction};
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::lists::row::row::Outline;
use ds::focus::request::use_focus_request;
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: &str| log.with_mut(|log| log.push(entry.to_owned()));
    let actions = vec![StripAction {
        id: ActionId("archive".to_string()),
        icon: Icon::Archive,
        label: "Archive".to_string(),
        fly: "Archive → out of Inbox".to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    }];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:400px",
                onkeydown: move |event: KeyboardEvent| note(&format!("key:{}", event.key())),
                div { class: "row", style: "display:flex; height:40px",
                    onclick: move |_| note("open"),
                    HoverStrip {
                        actions,
                        shown: Shown::Visible,
                        on_press: move |id: ActionId| note(&format!("press:{}", id.0)),
                    }
                }
                div { class: "tree", style: "width:260px",
                    Row {
                        title: "Projects",
                        leading: RowLeading::Icon(Icon::Folder),
                        outline: Outline::Branch(Shown::Visible),
                        on_toggle: move |_| note("toggle:projects"),
                        onclick: move |_: Press| note("select"),
                        accessory: Accessory::Slot(rsx! {
                            Button {
                                bezel: Bezel::Toolbar, image: ImagePosition::Only,
                                icon: Icon::Ellipsis,
                                label: "More",
                                onclick: move |_: Press| note("more"),
                            }
                        }),
                        Row { title: "Quire", outline: Outline::Leaf }
                    }
                }
                div { class: "blank", style: "height:20px" }
                p { class: "log", {log().join(",")} }
            }
        }
    }
}

fn harness(fallback: FocusFallback) -> Harness {
    let mut harness = Harness::new(
        Page,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_focus_fallback(fallback),
    );
    harness.advance(Duration::from_millis(50));
    harness
}

/// Click the element `selector` names, at its centre.
fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out:\n{}", harness.html()));
    harness.send(Input::click(at));
}

const STRIP_BUTTON: &str = ".row .ds-strip .ds-button";
const TRIANGLE: &str = ".tree .ds-row-disclosure[*|data-outline=branch]";
const TRAILING: &str = ".tree .ds-row-trailing .ds-button";

/// The press was the control's own (the log says so), and then the keyboard is on `focused`
/// and nowhere else: a key reaches `.app` through it.
fn pressed_focuses(selector: &str, logged: &str, focused: &str) {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, selector);
    settle_until(&mut harness, |harness| {
        harness.focus_of(focused) == FocusState::Focused
    });
    assert_eq!(
        harness.focus_of("html"),
        FocusState::Unfocused,
        "the keyboard is not on html"
    );
    harness.advance(Duration::from_millis(100));
    assert_eq!(
        harness.focus_of(focused),
        FocusState::Focused,
        "{focused} kept the keyboard"
    );
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some(format!("{logged},key:j").as_str()),
        "the press stayed the control's and the key reached .app"
    );
}

#[test]
fn a_strip_button_click_focuses_the_button() {
    pressed_focuses(STRIP_BUTTON, "press:archive", STRIP_BUTTON);
}

#[test]
fn a_disclosure_triangle_click_leaves_the_keyboard_on_the_app() {
    pressed_focuses(TRIANGLE, "toggle:projects", ".app");
}

#[test]
fn a_trailing_slot_click_focuses_the_button_in_it() {
    pressed_focuses(TRAILING, "more", TRAILING);
}

/// A kept click moves the keyboard off `.app` onto the pressed control, as a browser would.
#[test]
fn a_kept_click_takes_the_keyboard_from_the_app() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".blank");
    settle_until(&mut harness, |harness| {
        harness.focus_of(".app") == FocusState::Focused
    });
    click(&mut harness, TRAILING);
    settle_until(&mut harness, |harness| {
        harness.focus_of(TRAILING) == FocusState::Focused
    });
}

/// The negative control: under Blitz's own behaviour a kept click leaves the keyboard nowhere,
/// as it did for every one of them before.
#[test]
fn under_blitz_default_a_kept_click_leaves_the_keyboard_nowhere() {
    for selector in [STRIP_BUTTON, TRIANGLE, TRAILING] {
        let mut harness = harness(FocusFallback::BlitzDefault);
        click(&mut harness, selector);
        harness.advance(Duration::from_millis(200));
        assert!(
            harness.focus_of(selector) == FocusState::Unfocused
                && harness.focus_of(".app") == FocusState::Unfocused,
            "{selector}: Blitz's own default gives no control the keyboard"
        );
    }
}

/// One folder being renamed, another beside it; `.app` logs its pointer-ups, as mailo's ends a
/// drag there.
#[allow(non_snake_case)]
fn Renaming() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: &str| log.with_mut(|log| log.push(entry.to_owned()));
    let request = use_focus_request().with_select_all();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:300px",
                onpointerup: move |_| note("up"),
                div { class: "tree", style: "width:260px",
                    Row {
                        title: "Projects",
                        content: rsx! {
                            span { onclick: move |event: MouseEvent| event.stop_propagation(),
                                TextField { bezel: FieldBezel::Plain, label: "Rename folder", value: "Projects",
                                    focus: FieldFocus::Controlled(request),
                                    oninput: |_: String| {},
                                }
                            }
                        },
                        leading: RowLeading::Icon(Icon::Folder),
                        outline: Outline::Branch(Shown::Hidden),
                        on_toggle: move |_| note("toggle:projects"),
                        span { "inside" }
                    }
                    Row {
                        title: "Archive",
                        outline: Outline::Branch(Shown::Hidden),
                        on_toggle: move |_| note("toggle:archive"),
                        onclick: move |_: Press| note("select"),
                        span { "inside" }
                    }
                }
                p { class: "log", {log().join(",")} }
            }
        }
    }
}

const FIELD: &str = ".ds-row-words input";

fn renaming() -> Harness {
    let mut harness = Harness::new(
        Renaming,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    settle_until(&mut harness, |harness| {
        harness.focus_of(FIELD) == FocusState::Focused
    });
    harness
}

/// A press in the editing slot's field reaches an ancestor's pointer-up (the slot no longer
/// fences it), and still neither toggles nor selects the row.
#[test]
fn a_press_in_the_rename_field_reaches_the_apps_pointerup() {
    let mut harness = renaming();
    click(&mut harness, FIELD);
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.text_of(".log").as_deref(), Some("up"));
    assert_eq!(
        harness.focus_of(FIELD),
        FocusState::Focused,
        "the field kept the keyboard"
    );
}

/// A kept click with its default prevented cannot blur a field, so a field that has the
/// keyboard keeps it: the rename is not left without its field hearing so.
#[test]
fn a_field_with_the_keyboard_keeps_it_through_a_kept_click() {
    let mut harness = renaming();
    click(
        &mut harness,
        ".tree > .ds-row-branch:nth-child(2) .ds-row-disclosure",
    );
    harness.advance(Duration::from_millis(200));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("up,toggle:archive")
    );
    assert_eq!(
        harness.focus_of(FIELD),
        FocusState::Focused,
        "the field kept the keyboard"
    );
}
