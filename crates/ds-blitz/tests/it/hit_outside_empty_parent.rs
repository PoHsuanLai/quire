//! A hit test finds an absolutely positioned child drawn outside a parent box that has no height
//! (mailo's send pill inside `.send-at`): the parent's own style varies, the child is always
//! `position:absolute; bottom:0` with a real size, in a footer behind it.

use dioxus::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 200,
    scale_percent: 100,
};

/// The footer's style and the zero-height parent's, per variant.
const VARIANTS: &[(&str, &str)] = &[
    ("", "position:relative; height:0; width:100px"),
    (
        "",
        "position:relative; display:inline-block; height:0; width:100px",
    ),
    (
        "display:flex; align-items:flex-end",
        "position:relative; height:0; width:100px",
    ),
    ("", "position:relative; z-index:1; height:0; width:100px"),
    ("", "position:relative; z-index:1; height:1px; width:100px"),
    ("", "position:relative; z-index:1; height:30px; width:100px"),
    (
        "",
        "position:relative; transform:translateZ(0); height:0; width:100px",
    ),
    (
        "overflow:hidden",
        "position:relative; height:0; width:100px",
    ),
];

thread_local! {
    static VARIANT: Cell<usize> = const { Cell::new(0) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let (footer, parent) = VARIANTS[VARIANT.with(Cell::get)];
    let mut hits = use_signal(|| 0u32);
    rsx! {
        div { style: "padding:20px; width:260px",
            div { id: "footer", style: "height:60px; background:#ccc; margin-top:60px; {footer}",
                onclick: move |_| hits += 100,
                div { id: "parent", style: "{parent}",
                    div { id: "pill", style: "position:absolute; bottom:0; width:80px; height:30px; background:#36f",
                        onclick: move |event: MouseEvent| {
                            event.stop_propagation();
                            hits += 1;
                        },
                    }
                }
            }
            p { class: "log", "{hits}" }
        }
    }
}

/// What the log says after a click on the pill's centre, in variant `variant`.
fn click_pill(variant: usize) -> Option<String> {
    VARIANT.with(|slot| slot.set(variant));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    let pill = harness.rect("#pill").expect("the pill is laid out");
    assert!(pill.size.height.0 > 20.0, "the pill has its size");
    let at = harness.centre("#pill").expect("the pill's centre");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(50));
    harness.text_of(".log")
}

/// The variants at `which` (indices into [`VARIANTS`]) all hit the pill.
fn assert_hit(which: &[usize]) {
    for &variant in which {
        assert_eq!(
            click_pill(variant).as_deref(),
            Some("1"),
            "variant {variant}: {:?}",
            VARIANTS[variant]
        );
    }
}

#[test]
fn a_click_on_an_absolute_child_outside_a_zero_height_parent_hits_the_child() {
    assert_hit(&[0, 1, 2, 5, 6, 7]);
}

#[test]
fn a_click_on_an_absolute_child_outside_a_zero_height_stacking_parent_hits_the_child() {
    assert_hit(&[3, 4]);
}
