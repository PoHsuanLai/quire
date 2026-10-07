//! A click at the centre of a `position:absolute` button inside a `position:relative` parent with
//! padding reaches the button: its client rect is right, and so must be its hit.

use dioxus::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 200,
    scale_percent: 100,
};

/// The parent's style, per variant.
const VARIANTS: &[&str] = &[
    "position:relative; padding:12px 16px; width:200px; height:60px",
    "position:relative; padding:12px 16px",
    "position:relative; padding:12px 16px; width:200px; height:60px; z-index:1",
    "position:relative; padding:12px 16px; width:200px; height:60px; overflow:hidden",
    "position:relative; padding:12px 16px; width:200px; height:60px; display:flex",
];

thread_local! {
    static VARIANT: Cell<usize> = const { Cell::new(0) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let parent = VARIANTS[VARIANT.with(Cell::get)];
    let mut hits = use_signal(|| 0u32);
    rsx! {
        div { style: "padding:20px; width:260px",
            div { id: "parent", style: "{parent}",
                button {
                    id: "pill",
                    style: "position:absolute; left:60px; top:24px; width:52px; height:22px",
                    onclick: move |_| hits += 1,
                    "Go"
                }
            }
            p { class: "log", "{hits}" }
        }
    }
}

fn click_button(variant: usize) -> Option<String> {
    VARIANT.with(|slot| slot.set(variant));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    let at = harness.centre("#pill").expect("the button's centre");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(50));
    harness.text_of(".log")
}

#[test]
fn a_click_on_an_absolute_button_in_a_padded_relative_parent_hits_the_button() {
    for (variant, style) in VARIANTS.iter().enumerate() {
        assert_eq!(
            click_button(variant).as_deref(),
            Some("1"),
            "variant {variant}: {style}"
        );
    }
}
