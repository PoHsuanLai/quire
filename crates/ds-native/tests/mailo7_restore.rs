//! mailo gaps 7, item 2: a click's focus restore skips an element the click removed. mailo's
//! "Show images" button removes itself on press; the click's fallback had found that button (a
//! `button` is focusable) and focused it a frame later, after it had left the document, so the
//! keys that followed reached nothing. The restore now checks each remembered candidate when it
//! focuses, and falls to the next focusable ancestor (`.app[tabindex]`).

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonVariant, Ds, Key, Material, Press};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 240,
    scale_percent: 100,
};

/// Whether the banner's button removes itself when pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnPress {
    /// It goes, as "Show images" does once the images are shown.
    Leaves,
    /// It stays: the control.
    Stays,
}

#[component]
fn Page(on_press: OnPress) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut shown = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "app", tabindex: "0", style: "padding:20px; width:300px",
                onkeydown: move |event: KeyboardEvent| log.with_mut(|log| log.push(format!("key:{}", event.key()))),
                div { class: "banner", style: "display:flex; height:40px",
                    if shown() {
                        Button { variant: ButtonVariant::Secondary, label: "Show images",
                            onclick: move |_: Press| {
                                log.with_mut(|log| log.push("show".to_owned()));
                                if on_press == OnPress::Leaves {
                                    shown.set(false);
                                }
                            },
                        }
                    }
                }
                p { class: "log", {log().join(",")} }
            }
        }
    }
}

fn leaves() -> Element {
    rsx! { Page { on_press: OnPress::Leaves } }
}

fn stays() -> Element {
    rsx! { Page { on_press: OnPress::Stays } }
}

fn harness(on_press: OnPress) -> Harness {
    let app = match on_press {
        OnPress::Leaves => leaves,
        OnPress::Stays => stays,
    };
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(50));
    harness
}

fn press_show(harness: &mut Harness) {
    let at = harness
        .centre(".banner .ds-button")
        .unwrap_or_else(|| panic!("the button is not laid out:\n{}", harness.html()));
    harness.click(at);
}

#[test]
fn a_button_that_removes_itself_leaves_the_keyboard_on_its_focusable_ancestor() {
    let mut harness = harness(OnPress::Leaves);
    press_show(&mut harness);
    ds_native::harness::settle_until(&mut harness, |harness| harness.is_focused(".app"));
    assert_eq!(harness.count(".banner .ds-button"), 0, "the button left");
    harness.key(Key::Char('j'));
    assert_eq!(harness.text_of(".log").as_deref(), Some("show,key:j"));
}

#[test]
fn a_button_that_stays_keeps_the_keyboard() {
    let mut harness = harness(OnPress::Stays);
    press_show(&mut harness);
    ds_native::harness::settle_until(&mut harness, |harness| {
        harness.is_focused(".banner .ds-button")
    });
    harness.key(Key::Char('j'));
    assert_eq!(harness.text_of(".log").as_deref(), Some("show,key:j"));
}
