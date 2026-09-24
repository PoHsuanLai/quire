//! Where `Harness::click` reaches a Button (FINDINGS "Gallery fixes B" observed a Button
//! followed only by an `if` placeholder never receiving the click; FINDINGS "Polish pass" has
//! the reproduction's outcome).
//!
//! The placeholder is not the cause. A Button is `display:inline-flex`, an atomic inline: when
//! its parent holds nothing but inline content (the Button alone, the Button and an `if`
//! placeholder, or the Button and some inline text), the parent is an inline formatting
//! context and blitz-dom's `Node::hit` returns the parent, not the Button, so the click goes
//! to the parent. A block after the Button (the anchor test's caption) or a flex parent puts
//! the Button in a box of its own and the click lands. The same hit test serves a real window,
//! so this is blitz-dom's (at the pinned rev), not the harness's; ds-native neither hit-tests
//! nor routes clicks itself.

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonVariant, Ds, Material, Switch};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// A Button that shows whether it was clicked, and nothing after it but an `if` placeholder.
#[component]
fn Clickable() -> Element {
    let mut state = use_signal(|| Switch::Off);
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            label: format!("{:?}", state()),
            onclick: move |_| state.set(Switch::On),
        }
        if state() == Switch::On {
            span { "Opened" }
        }
    }
}

#[allow(non_snake_case)]
fn InlineApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px", Clickable {} }
        }
    }
}

#[allow(non_snake_case)]
fn InlineTextApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px", Clickable {} span { "after it, inline" } }
        }
    }
}

#[allow(non_snake_case)]
fn BlockAfterApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px", Clickable {} p { "after it, a block" } }
        }
    }
}

#[allow(non_snake_case)]
fn FlexApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px; display:flex", Clickable {} }
        }
    }
}

/// Click the Button's centre and read its label: `On` once the click reached it.
fn label_after_click(app: fn() -> Element) -> String {
    let mut harness = Harness::new(app, VIEW);
    let at = harness
        .centre(".ds-button")
        .expect("the button is on screen");
    harness.click(at);
    harness.advance(Duration::from_millis(50));
    harness.text_of(".ds-button").unwrap_or_default()
}

#[test]
fn a_click_reaches_a_button_followed_by_a_block() {
    assert_eq!(label_after_click(BlockAfterApp), "On");
}

#[test]
fn a_click_reaches_a_button_in_a_flex_row_with_only_a_placeholder_after_it() {
    assert_eq!(label_after_click(FlexApp), "On");
}

#[test]
#[ignore = "blitz-dom at e99fbdbd hits the parent of an atomic inline in an inline formatting \
            context, not the inline-flex Button: the click never reaches it (FINDINGS Polish pass)"]
fn a_click_reaches_a_button_in_an_inline_formatting_context() {
    assert_eq!(
        label_after_click(InlineApp),
        "On",
        "only a placeholder after it"
    );
    assert_eq!(
        label_after_click(InlineTextApp),
        "On",
        "inline text after it"
    );
}
