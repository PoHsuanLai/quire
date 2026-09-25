//! Native focus, keep-focus: a click on nothing focusable leaves the keyboard on the nearest
//! focusable ancestor (mailo's `.app[tabindex]`), as a browser does, where Blitz would clear it;
//! `FocusFallback::BlitzDefault` keeps Blitz's behaviour. A field the click leaves still hears
//! its blur, the ancestor hears no spurious one, a double click still arrives, and an edit
//! surface's own focus wins inside it.

use dioxus::prelude::*;
use ds::{Appearance, Ds, EditSurface, InputVariant, Key, Material, TextInput};
use ds_native::{FocusFallback, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A key-handling shell around plain text, a field and an edit surface, logging what it hears.
#[allow(non_snake_case)]
fn Shell() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |line: String| log.with_mut(|log| log.push(line));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:300px",
                onkeydown: move |event: KeyboardEvent| note(format!("key:{}", event.key())),
                onblur: move |_| note("app-blur".to_owned()),
                p { class: "plain", style: "margin:0; height:24px",
                    ondoubleclick: move |_| note("dblclick".to_owned()),
                    "Plain text, nothing focusable"
                }
                div { class: "field", style: "display:flex; margin-top:12px",
                    TextInput { variant: InputVariant::Boxed, label: "Find", value: "",
                        oninput: |_| {},
                        onblur: move |()| note("field-blur".to_owned()),
                    }
                }
                div { style: "margin-top:12px; height:40px",
                    EditSurface { id: "editor", on_input: |_| {},
                        p { class: "inside", style: "margin:0", "Editable text" }
                    }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn harness(fallback: FocusFallback) -> Harness {
    let mut harness = Harness::with_config(
        Shell,
        HarnessConfig::new(VIEW).with_focus_fallback(fallback),
    );
    harness.advance(ms(50));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()));
    harness.click(at);
    // A frame: the fallback waits out a document busy with the click's own re-render.
    harness.advance(ms(60));
}

#[test]
fn a_click_on_plain_text_leaves_the_shell_focused_and_it_hears_the_next_key() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    assert!(harness.is_focused(".app"));
    harness.key(Key::Char('j'));
    assert_eq!(log(&harness), "key:j");
}

#[test]
fn with_blitz_default_the_click_clears_the_focus() {
    let mut harness = harness(FocusFallback::BlitzDefault);
    click(&mut harness, ".plain");
    assert!(!harness.is_focused(".app"));
    harness.key(Key::Char('j'));
    assert_eq!(log(&harness), "");
}

#[test]
fn a_second_click_keeps_the_shell_focused_without_a_blur() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    harness.advance(ms(600));
    click(&mut harness, ".plain");
    assert!(harness.is_focused(".app"));
    assert_eq!(log(&harness), "", "the shell heard no blur");
}

#[test]
fn a_double_click_still_arrives() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    click(&mut harness, ".plain");
    assert!(harness.is_focused(".app"));
    assert_eq!(log(&harness), "dblclick");
}

#[test]
fn a_field_the_click_leaves_hears_its_blur_and_the_shell_takes_the_keyboard() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".field input");
    assert!(harness.is_focused(".field input"));
    click(&mut harness, ".plain");
    assert!(harness.is_focused(".app"));
    assert_eq!(log(&harness), "field-blur");
}

#[test]
fn inside_an_edit_surface_its_own_focus_wins() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    click(&mut harness, ".inside");
    assert!(harness.is_focused("#editor"));
    assert!(!harness.is_focused(".app"));
}
