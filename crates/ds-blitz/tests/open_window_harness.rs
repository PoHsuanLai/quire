//! `ds_blitz::open_window` and `use_window_handle` outside a window `launch` runs: the harness has
//! no event loop to ask, so the call answers `OpenWindowError::NoHost` and opens nothing, and the
//! hook finds no handle (a live run of the
//! `second_window` example is the proof for a real window; FINDINGS "File drops and a second
//! window").

use dioxus::prelude::*;
use ds_blitz::{OpenWindowError, WindowSize, WindowSpec, open_window, use_window_handle};
use ds_harness::{Clock, Harness, HarnessConfig, Query, Viewport};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 100,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Other() -> Element {
    rsx! { p { "another window" } }
}

#[allow(non_snake_case)]
fn Asks() -> Element {
    let answer = use_hook(|| {
        match open_window(WindowSpec::new("Other", WindowSize::new(100, 100)), Other) {
            Ok(_) => "opened".to_owned(),
            Err(OpenWindowError::NoHost) => "no host".to_owned(),
        }
    });
    rsx! { p { class: "answer", "{answer}" } }
}

#[test]
fn the_harness_has_no_event_loop_to_open_a_window_on() {
    let harness = Harness::new(Asks, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(harness.text_of(".answer").as_deref(), Some("no host"));
}

#[allow(non_snake_case)]
fn Reads() -> Element {
    let found = use_window_handle().is_some();
    rsx! { p { class: "found", "{found}" } }
}

#[test]
fn the_harness_window_has_no_handle_to_raise() {
    let harness = Harness::new(Reads, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(harness.text_of(".found").as_deref(), Some("false"));
}
