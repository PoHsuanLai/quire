//! A controlled focus that selects the field's whole value once the caret
//! lands (a rename field opened on the old name), and the public `focus_soon` an app uses for
//! its own element.

use dioxus::prelude::*;
use ds::focus::request::use_focus_request;
use ds::focus::soon::focus_soon;
use ds::prelude::*;
use ds_harness::{Driver, FocusState, Harness, Input, Query, Viewport};
use std::rc::Rc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 240,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A rename field on "Archive", focused with select-all on mount and by the button; a second
/// field to move away to; the value echoed below.
#[allow(non_snake_case)]
fn Rename() -> Element {
    let mut name = use_signal(|| "Archive".to_owned());
    let request = use_focus_request().with_select_all();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "name", style: "display:flex",
                    TextField { label: "Name", value: name(),
                        focus: FieldFocus::Controlled(request), oninput: move |value| name.set(value) }
                }
                div { id: "other", style: "display:flex",
                    TextField { label: "Other", value: "", oninput: |_| {} }
                }
                div { id: "again", style: "display:flex",
                    Button { label: "Rename",
                        onclick: move |_| request.request() }
                }
            }
            p { class: "name", {name()} }
        }
    }
}

/// The same field, controlled but without select-all.
#[allow(non_snake_case)]
fn Plain() -> Element {
    let request = use_focus_request();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "name", style: "display:flex; width:300px; padding:12px",
                TextField { label: "Name", value: "Archive",
                    focus: FieldFocus::Controlled(request), oninput: |_| {} }
            }
        }
    }
}

#[test]
fn a_controlled_focus_with_select_all_selects_the_whole_value() {
    let mut harness = Harness::new(Rename, VIEW);
    harness.advance(ms(100));
    assert_eq!(harness.focus_of("#name input"), FocusState::Focused);
    assert_eq!(
        harness.selected_text("#name input").as_deref(),
        Some("Archive")
    );
    harness.send(Input::key(ShortcutKey::Char('X')));
    harness.advance(ms(20));
    assert_eq!(
        harness.text_of(".name").as_deref(),
        Some("X"),
        "typing replaced it"
    );
}

#[test]
fn a_request_after_moving_away_selects_it_again() {
    let mut harness = Harness::new(Rename, VIEW);
    harness.advance(ms(100));
    let other = harness.centre("#other input").expect("the other field");
    harness.send(Input::click(other));
    harness.advance(ms(50));
    assert_eq!(harness.focus_of("#other input"), FocusState::Focused);
    let again = harness.centre("#again .ds-button").expect("the button");
    harness.send(Input::click(again));
    harness.advance(ms(100));
    assert_eq!(harness.focus_of("#name input"), FocusState::Focused);
    assert_eq!(
        harness.selected_text("#name input").as_deref(),
        Some("Archive")
    );
}

#[test]
fn a_controlled_focus_without_select_all_selects_nothing() {
    let mut harness = Harness::new(Plain, VIEW);
    harness.advance(ms(100));
    assert_eq!(harness.focus_of("#name input"), FocusState::Focused);
    assert_eq!(harness.selected_text("#name input"), None);
}

/// An app's own focusable element, focused through the public seam from a button.
#[allow(non_snake_case)]
fn Shell() -> Element {
    let mut shell = use_signal(|| None::<Rc<MountedData>>);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { class: "app", tabindex: "0", onmounted: move |event| shell.set(Some(event.data())),
                div { id: "back", style: "display:flex; width:200px",
                    Button { label: "Back",
                        onclick: move |_| if let Some(element) = shell() { focus_soon(element) } }
                }
            }
        }
    }
}

#[test]
fn an_app_focuses_its_own_element_through_focus_soon() {
    let mut harness = Harness::new(Shell, VIEW);
    assert_eq!(harness.focus_of(".app"), FocusState::Unfocused);
    let back = harness.centre("#back .ds-button").expect("the button");
    harness.send(Input::click(back));
    harness.advance(ms(50));
    assert_eq!(
        harness.focus_of(".app"),
        FocusState::Focused,
        "{}",
        harness.html()
    );
}
