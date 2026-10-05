//! The keyboard after the element that had it leaves the document. Blitz
//! resets the focus to nowhere when the focused node is removed, so after a quire Menu closed on
//! Escape (its panel had the keyboard) or a field removed itself on Enter, keys reached nothing
//! and mailo's `.app[tabindex]` stopped hearing them. Under `FocusFallback::Ancestor` the host
//! now focuses the removed element's nearest focusable ancestor, or (for a floating menu's
//! panel, whose ancestors are the overlay layer) the element focused before it; a menu anchored
//! to a mounted element gives the keyboard back to that element itself.

use dioxus::prelude::*;
use ds::base::press::Press;
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::root::common::Common;
use ds_blitz::FocusFallback;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn entries() -> Vec<MenuItem<u8>> {
    vec![MenuItem::new(0, "Rename"), MenuItem::new(1, "Delete")]
}

/// A shell whose keys open a menu ("m") and are logged; its "More" button opens the same menu.
/// The menu is anchored at a point, or to the button's element.
#[component]
fn MenuPage(anchored: Anchored) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut open = use_signal(|| false);
    let mut button = use_signal(|| None::<MountedRef>);
    let anchor = match (anchored, button()) {
        (Anchored::ToButton, Some(element)) => Anchor::Mounted(element),
        _ => Anchor::Point(Point {
            x: Px(40.0),
            y: Px(80.0),
        }),
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:300px; height:200px",
                onkeydown: move |event: KeyboardEvent| {
                    log.with_mut(|log| log.push(format!("key:{}", event.key())));
                    if event.key() == dioxus::prelude::Key::Character("m".to_owned()) {
                        open.set(true);
                    }
                },
                div { class: "bar", style: "display:flex; height:32px",
                    Button { common: Common { mounted: Some(EventHandler::new(move |event: MountedEvent| button.set(Some(MountedRef(event.data()))))), ..Common::default() }, label: "More",
                        onclick: move |_: Press| open.set(true),
                    }
                }
                p { class: "log", {log().join(",")} }
            }
            if open() {
                Menu::<u8> {
                    placement: MenuPlacement::Bar,
                    anchor,
                    items: entries(),
                    onpick: |_| {},
                    onclose: move |()| open.set(false),
                }
            }
        }
    }
}

/// What the page's menu is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Anchored {
    /// A point: the menu knows no element to give the keyboard back to.
    AtPoint,
    /// The More button's element.
    ToButton,
}

fn at_point() -> Element {
    rsx! { MenuPage { anchored: Anchored::AtPoint } }
}

fn to_button() -> Element {
    rsx! { MenuPage { anchored: Anchored::ToButton } }
}

/// A shell holding a pane (itself focusable) holding a field that removes itself on Enter, as
/// a rename field does when it commits.
#[allow(non_snake_case)]
fn FieldPage() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut editing = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:300px",
                onkeydown: move |event: KeyboardEvent| log.with_mut(|log| log.push(format!("key:{}", event.key()))),
                div { class: "pane", tabindex: "0", style: "padding:8px",
                    div { class: "wrap", style: "display:flex; height:32px",
                        if editing() {
                            TextField { label: "Name", value: "Inbox",
                                oninput: |_| {},
                                onkey: move |event: KeyboardEvent| {
                                    if event.key() == dioxus::prelude::Key::Enter {
                                        editing.set(false);
                                    }
                                },
                            }
                        }
                    }
                }
                p { class: "log", {log().join(",")} }
            }
        }
    }
}

fn harness(app: fn() -> Element, fallback: FocusFallback) -> Harness {
    let mut harness = Harness::new(
        app,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_focus_fallback(fallback),
    );
    harness.advance(ms(50));
    harness
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out:\n{}", harness.html()));
    harness.send(Input::click(at));
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// Open the menu with `open`, wait for its panel to take the keyboard, close it with Escape and
/// wait for it to leave.
fn open_and_escape(harness: &mut Harness, open: impl FnOnce(&mut Harness)) {
    open(harness);
    settle_until(harness, |harness| {
        harness.focus_of(".ds-menu") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Escape));
    settle_until(harness, |harness| harness.count(".ds-menu") == 0);
}

#[test]
fn a_menu_opened_from_a_button_and_closed_with_escape_leaves_the_keyboard_in_the_app() {
    let mut harness = harness(at_point, FocusFallback::Ancestor);
    open_and_escape(&mut harness, |harness| click(harness, ".bar .ds-button"));
    settle_until(&mut harness, |harness| {
        harness.focus_of(".bar .ds-button") == FocusState::Focused
            || harness.focus_of(".app") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:j");
}

#[test]
fn a_menu_opened_by_a_key_on_the_app_gives_the_keyboard_back_to_the_app() {
    let mut harness = harness(at_point, FocusFallback::Ancestor);
    click(&mut harness, ".log");
    settle_until(&mut harness, |harness| {
        harness.focus_of(".app") == FocusState::Focused
    });
    open_and_escape(&mut harness, |harness| {
        harness.send(Input::key(ShortcutKey::Char('m')))
    });
    settle_until(&mut harness, |harness| {
        harness.focus_of(".app") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:m,key:j");
}

#[test]
fn a_menu_anchored_to_an_element_gives_the_keyboard_back_to_it() {
    // Opened by a key on the app, so the button never had the keyboard: only the menu's own
    // hand-back to its anchor puts it there (the host's fallback would pick the app).
    let mut harness = harness(to_button, FocusFallback::Ancestor);
    click(&mut harness, ".log");
    settle_until(&mut harness, |harness| {
        harness.focus_of(".app") == FocusState::Focused
    });
    open_and_escape(&mut harness, |harness| {
        harness.send(Input::key(ShortcutKey::Char('m')))
    });
    settle_until(&mut harness, |harness| {
        harness.focus_of(".bar .ds-button") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:m,key:j");
}

#[test]
fn with_blitz_default_the_focus_stays_nowhere_after_escape() {
    let mut harness = harness(at_point, FocusFallback::BlitzDefault);
    open_and_escape(&mut harness, |harness| click(harness, ".bar .ds-button"));
    harness.advance(ms(100));
    assert_eq!(harness.focus_of(".app"), FocusState::Unfocused);
    assert_eq!(harness.focus_of(".bar .ds-button"), FocusState::Unfocused);
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "");
}

#[test]
fn a_field_removed_by_its_own_handler_leaves_the_keyboard_on_its_nearest_focusable_ancestor() {
    let mut harness = harness(FieldPage, FocusFallback::Ancestor);
    click(&mut harness, ".wrap input");
    settle_until(&mut harness, |harness| {
        harness.focus_of(".wrap input") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Enter));
    settle_until(&mut harness, |harness| {
        harness.focus_of(".pane") == FocusState::Focused
    });
    assert_eq!(harness.count(".wrap input"), 0, "the field left");
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:Enter,key:j");
}

#[test]
fn with_blitz_default_a_removed_field_leaves_the_focus_nowhere() {
    let mut harness = harness(FieldPage, FocusFallback::BlitzDefault);
    click(&mut harness, ".wrap input");
    settle_until(&mut harness, |harness| {
        harness.focus_of(".wrap input") == FocusState::Focused
    });
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(100));
    assert_eq!(harness.count(".wrap input"), 0, "the field left");
    assert_eq!(harness.focus_of(".pane"), FocusState::Unfocused);
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:Enter");
}
