//! A toast pushed from a sheet's own button keeps clear of that button (design/30 section 2.9):
//! while a sheet is up the toast sits at the top right, so a second press on the button reaches
//! the button and not the toast (whose hold pauses under a still pointer). With no sheet it stays
//! at the bottom right.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 900,
    height: 640,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A sheet whose bottom-right button pushes a toast and counts its presses.
#[allow(non_snake_case)]
fn WithSheet() -> Element {
    let toasts = use_toasts();
    let mut presses = use_signal(|| 0u32);
    rsx! {
        Sheet { label: "Add studio", onclose: |_| {},
            div { style: "height:320px" }
            div { style: "display:flex; justify-content:flex-end; padding:20px",
                Button {
                    label: "Add",
                    common: Common { id: Some("add".to_string()), ..Common::default() },
                    onclick: move |_| {
                        presses += 1;
                        toasts.push(format!("Added {}", presses()), None);
                    },
                }
            }
        }
        p { class: "presses", "{presses}" }
    }
}

/// The same button with no sheet around it.
#[allow(non_snake_case)]
fn NoSheet() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button {
            label: "Add",
            common: Common { id: Some("add".to_string()), ..Common::default() },
            onclick: move |_| toasts.push("Added".into(), None),
        }
    }
}

#[allow(non_snake_case)]
fn SheetPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            WithSheet {}
        }
    }
}

#[allow(non_snake_case)]
fn PlainPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            NoSheet {}
        }
    }
}

fn start(page: fn() -> Element) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(600));
    harness
}

#[test]
fn a_toast_over_a_sheet_sits_at_the_top_and_the_button_takes_the_second_press() {
    let mut harness = start(SheetPage);
    let button = harness.rect("#add").expect("the sheet's button");
    let at = harness.centre("#add").expect("its centre");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-place").as_deref(),
        Some("top")
    );
    let toast = harness.rect(".ds-toast").expect("the toast");
    assert!(
        toast.origin.y.0 + toast.size.height.0 < button.origin.y.0,
        "the toast {toast:?} is clear of the button {button:?}"
    );
    harness.send(Input::click(at));
    harness.advance(ms(100));
    assert_eq!(harness.text_of(".presses").as_deref(), Some("2"));
}

#[test]
fn with_no_sheet_the_toast_stays_at_the_bottom() {
    let mut harness = start(PlainPage);
    let at = harness.centre("#add").expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-place").as_deref(),
        Some("bottom")
    );
    let toast = harness.rect(".ds-toast").expect("the toast");
    assert!(toast.origin.y.0 > VIEW.height as f32 / 2.0, "{toast:?}");
}
