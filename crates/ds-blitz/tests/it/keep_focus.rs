//! Native focus, keep-focus: a click on nothing focusable leaves the keyboard on the nearest
//! focusable ancestor (mailo's `.app[tabindex]`), as a browser does, where Blitz would clear it;
//! `FocusFallback::BlitzDefault` keeps Blitz's behaviour. A field the click leaves still hears
//! its blur, the ancestor hears no spurious one, a double click still arrives, and an edit
//! surface's own focus wins inside it. A shell surface's panel is the same: a press on its empty
//! space keeps the keyboard on the panel, so Escape still reaches the panel's handler.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
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
                    TextField { label: "Find", value: "",
                        oninput: |_| {},
                        onblur: move |()| note("field-blur".to_owned()),
                    }
                }
                div { style: "margin-top:12px; height:40px",
                    EditSurface { common: Common { id: Some("editor".to_string()), ..Common::default() }, on_input: |_| {},
                        p { class: "inside", style: "margin:0", "Editable text" }
                    }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// A shell surface's shape: a panel that closes on Escape, with a button inside and room around it.
#[allow(non_snake_case)]
fn Panel() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "panel", tabindex: "0", style: "padding:20px; width:300px; height:200px",
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape {
                        log.with_mut(|log| log.push("closed".to_owned()));
                    }
                },
                button { class: "inside", style: "height:30px", "Inside" }
                div { class: "empty", style: "height:100px" }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn panel(fallback: FocusFallback) -> Harness {
    let mut harness = Harness::new(
        Panel,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_focus_fallback(fallback),
    );
    harness.advance(ms(50));
    harness
}

fn harness(fallback: FocusFallback) -> Harness {
    harness_on(fallback, Clock::Virtual)
}

fn harness_on(fallback: FocusFallback, clock: Clock) -> Harness {
    let mut harness = Harness::new(
        Shell,
        HarnessConfig::new(VIEW)
            .with_clock(clock)
            .with_focus_fallback(fallback),
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
    harness.send(Input::click(at));
    // A frame: the fallback waits out a document busy with the click's own re-render.
    harness.advance(ms(60));
}

#[test]
fn a_click_on_plain_text_leaves_the_shell_focused_and_it_hears_the_next_key() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    assert!(harness.focus_of(".app"));
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "key:j");
}

#[test]
fn with_blitz_default_the_click_clears_the_focus() {
    let mut harness = harness(FocusFallback::BlitzDefault);
    click(&mut harness, ".plain");
    assert!(!harness.focus_of(".app"));
    harness.send(Input::key(ShortcutKey::Char('j')));
    assert_eq!(log(&harness), "");
}

#[test]
fn a_second_click_keeps_the_shell_focused_without_a_blur() {
    // On the wall clock, on purpose: Blitz decides a double click from its own wall-clock
    // `last_mousedown_time` (500 ms), which a virtual `advance` never lets pass. Only real time
    // passing can end the first click's window, and a loaded machine only stretches it.
    let mut harness = harness_on(FocusFallback::Ancestor, Clock::Wall);
    click(&mut harness, ".plain");
    harness.advance(ms(600));
    click(&mut harness, ".plain");
    assert!(harness.focus_of(".app"));
    assert_eq!(log(&harness), "", "the shell heard no blur");
}

#[test]
fn a_double_click_still_arrives() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    click(&mut harness, ".plain");
    assert!(harness.focus_of(".app"));
    assert_eq!(log(&harness), "dblclick");
}

#[test]
fn a_field_the_click_leaves_hears_its_blur_and_the_shell_takes_the_keyboard() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".field input");
    assert!(harness.focus_of(".field input"));
    click(&mut harness, ".plain");
    assert!(harness.focus_of(".app"));
    assert_eq!(log(&harness), "field-blur");
}

#[test]
fn inside_an_edit_surface_its_own_focus_wins() {
    let mut harness = harness(FocusFallback::Ancestor);
    click(&mut harness, ".plain");
    click(&mut harness, ".inside");
    assert!(harness.focus_of("#editor"));
    assert!(!harness.focus_of(".app"));
}

#[test]
fn a_press_on_a_panels_empty_space_keeps_the_keyboard_and_escape_still_closes_it() {
    let mut harness = panel(FocusFallback::Ancestor);
    // A plain button takes no focus in Blitz; the press on it leaves the panel as it was.
    click(&mut harness, ".inside");
    click(&mut harness, ".empty");
    assert!(harness.focus_of(".panel"));
    harness.send(Input::key(ShortcutKey::Escape));
    assert_eq!(log(&harness), "closed");
}
