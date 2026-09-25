//! G8 (Native focus): a field reached by its `FieldHandle` or by selector takes the keyboard,
//! selects its value, and hears `onfocus` once, though Blitz's focus write dispatches no event;
//! a handle's blur is heard once too; an unknown or unreadable selector is a typed error.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, FieldHandle, FocusError, InputVariant, Material, Select,
    TextInput, focus_by_selector, use_field_handle,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// How the page's buttons reach the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// Through the field's handle.
    Handle,
    /// Through `focus_by_selector(selector)`.
    Selector(&'static str),
}

/// Ask for the focus as `reach` says, logging a selector's outcome.
fn ask(reach: Reach, handle: FieldHandle, mut log: Signal<Vec<String>>) {
    match reach {
        Reach::Handle => handle.focus(Select::All),
        Reach::Selector(selector) => {
            spawn(async move {
                let outcome = match focus_by_selector(selector, Select::All).await {
                    Ok(()) => "found".to_owned(),
                    Err(FocusError::NoSuchElement { .. }) => "no-such-element".to_owned(),
                    Err(FocusError::BadSelector { .. }) => "bad-selector".to_owned(),
                    Err(other) => format!("{other:?}"),
                };
                log.with_mut(|log| log.push(outcome));
            });
        }
    }
}

/// A rename field on "Inbox" logging its focus and blur; "Go" reaches it as `reach` says, "Out"
/// blurs it through its handle.
#[component]
fn Page(reach: Reach) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let handle = use_field_handle();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { class: "rename", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "Name", value: "Inbox",
                        handle: Some(handle),
                        oninput: |_| {},
                        onfocus: move |()| log.with_mut(|log| log.push("focus".to_owned())),
                        onblur: move |()| log.with_mut(|log| log.push("blur".to_owned())),
                    }
                }
                div { id: "go", style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Go",
                        onclick: move |_| ask(reach, handle, log) }
                }
                div { id: "out", style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Out",
                        onclick: move |_| handle.blur() }
                }
                p { class: "mounted", if handle.element().is_some() { "mounted" } }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn ByHandle() -> Element {
    rsx! { Page { reach: Reach::Handle } }
}

#[allow(non_snake_case)]
fn BySelector() -> Element {
    rsx! { Page { reach: Reach::Selector(".rename input") } }
}

#[allow(non_snake_case)]
fn ByUnknown() -> Element {
    rsx! { Page { reach: Reach::Selector("#nothing-here") } }
}

#[allow(non_snake_case)]
fn ByGarbage() -> Element {
    rsx! { Page { reach: Reach::Selector("[[") } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn press(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()));
    harness.click(at);
}

fn reached(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(50));
    press(&mut harness, "#go button");
    harness.advance(ms(150));
    harness
}

#[test]
fn a_field_focused_by_its_handle_hears_it_once_with_its_value_selected() {
    let harness = reached(ByHandle);
    assert!(harness.is_focused(".rename input"));
    assert_eq!(log(&harness), "focus");
    assert_eq!(
        harness.selected_text(".rename input").as_deref(),
        Some("Inbox")
    );
}

#[test]
fn a_field_focused_by_selector_hears_it_once_with_its_value_selected() {
    let harness = reached(BySelector);
    assert!(harness.is_focused(".rename input"));
    assert_eq!(log(&harness), "focus,found");
    assert_eq!(
        harness.selected_text(".rename input").as_deref(),
        Some("Inbox")
    );
}

#[test]
fn a_handle_blur_is_heard_once_and_leaves_the_field() {
    let mut harness = reached(ByHandle);
    press(&mut harness, "#out button");
    harness.advance(ms(150));
    assert!(!harness.is_focused(".rename input"));
    assert_eq!(log(&harness), "focus,blur");
}

#[test]
fn the_handle_hands_out_the_mounted_element() {
    let mut harness = Harness::new(ByHandle, VIEW);
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".mounted").as_deref(), Some("mounted"));
}

#[test]
fn an_unknown_selector_is_no_such_element() {
    let mut harness = Harness::new(ByUnknown, VIEW);
    harness.advance(ms(50));
    press(&mut harness, "#go button");
    harness.advance(ms(1_000));
    assert_eq!(log(&harness), "no-such-element");
    assert!(!harness.is_focused(".rename input"));
}

#[test]
fn an_unreadable_selector_is_a_bad_selector() {
    let mut harness = Harness::new(ByGarbage, VIEW);
    harness.advance(ms(50));
    press(&mut harness, "#go button");
    harness.advance(ms(150));
    assert_eq!(log(&harness), "bad-selector");
}
