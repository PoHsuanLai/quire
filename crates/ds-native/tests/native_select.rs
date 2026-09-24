//! G6 (mailo Phase B): a controlled focus that selects the field's whole value once the caret
//! lands (a rename field opened on the old name), and the public `focus_soon` an app uses for
//! its own element.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, Focus, InputVariant, Key, Material, TextInput,
    focus_soon, use_focus_request,
};
use ds_native::{Harness, Viewport};
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
                    TextInput { variant: InputVariant::Boxed, label: "Name", value: name(),
                        focus: Focus::Controlled(request), oninput: move |value| name.set(value) }
                }
                div { id: "other", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "Other", value: "", oninput: |_| {} }
                }
                div { id: "again", style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Rename",
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
                TextInput { variant: InputVariant::Boxed, label: "Name", value: "Archive",
                    focus: Focus::Controlled(request), oninput: |_| {} }
            }
        }
    }
}

#[test]
fn a_controlled_focus_with_select_all_selects_the_whole_value() {
    let mut harness = Harness::new(Rename, VIEW);
    harness.advance(ms(100));
    assert!(harness.is_focused("#name input"));
    assert_eq!(
        harness.selected_text("#name input").as_deref(),
        Some("Archive")
    );
    harness.key(Key::Char('X'));
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
    harness.click(other);
    harness.advance(ms(50));
    assert!(harness.is_focused("#other input"));
    let again = harness.centre("#again .ds-button").expect("the button");
    harness.click(again);
    harness.advance(ms(100));
    assert!(harness.is_focused("#name input"));
    assert_eq!(
        harness.selected_text("#name input").as_deref(),
        Some("Archive")
    );
}

#[test]
fn a_controlled_focus_without_select_all_selects_nothing() {
    let mut harness = Harness::new(Plain, VIEW);
    harness.advance(ms(100));
    assert!(harness.is_focused("#name input"));
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
                    Button { variant: ButtonVariant::Secondary, label: "Back",
                        onclick: move |_| if let Some(element) = shell() { focus_soon(element) } }
                }
            }
        }
    }
}

#[test]
fn an_app_focuses_its_own_element_through_focus_soon() {
    let mut harness = Harness::new(Shell, VIEW);
    assert!(!harness.is_focused(".app"));
    let back = harness.centre("#back .ds-button").expect("the button");
    harness.click(back);
    harness.advance(ms(50));
    assert!(harness.is_focused(".app"), "{}", harness.html());
}
