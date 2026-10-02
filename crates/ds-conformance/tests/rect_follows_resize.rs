//! `use_rect` measures an element again when the host reports that the window changed size
//! (`WindowResized`): an element that follows the window reports where it is now, not where it
//! was when it mounted.

use dioxus::prelude::*;
use ds::host::measure::use_rect;
use ds::host::resized::WindowResized;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

/// A box whose width changes with the "window", and a button that stands in for the host: it
/// changes the width and then reports the resize.
#[allow(non_snake_case)]
fn App() -> Element {
    let resized = use_context_provider(WindowResized::new);
    let mut wide = use_signal(|| false);
    let probe = use_rect();
    let width = probe.rect().map_or(0.0, |rect| rect.size.width.0);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                class: "follower",
                style: if wide() { "width:300px; height:20px" } else { "width:100px; height:20px" },
                onmounted: move |event| probe.on_mounted(event),
            }
            button {
                class: "resize",
                style: "width:40px; height:20px",
                onclick: move |_| {
                    wide.set(true);
                    resized.bump();
                },
            }
            p { class: "measured", "{width}" }
        }
    }
}

#[test]
fn a_resize_report_measures_the_element_again() {
    let mut harness = Harness::new(App, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.text_of(".measured").as_deref(), Some("100"));
    let button = harness.centre(".resize").expect("the button");
    harness.send(Input::click(button));
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.text_of(".measured").as_deref(), Some("300"));
}
