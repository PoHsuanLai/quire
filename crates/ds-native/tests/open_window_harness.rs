//! `ds_native::open_window` outside a window `launch` runs: the harness has no event loop to ask,
//! so the call answers `OpenWindowError::NoHost` and opens nothing (a live run of the
//! `second_window` example is the proof for a real window; FINDINGS "File drops and a second
//! window").

use dioxus::prelude::*;
use ds_native::{Harness, OpenWindowError, Viewport, WindowSpec, open_window};

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
    let answer = use_hook(
        || match open_window(WindowSpec::new("Other", 100, 100), Other) {
            Ok(_) => "opened".to_owned(),
            Err(OpenWindowError::NoHost) => "no host".to_owned(),
        },
    );
    rsx! { p { class: "answer", "{answer}" } }
}

#[test]
fn the_harness_has_no_event_loop_to_open_a_window_on() {
    let harness = Harness::new(Asks, VIEW);
    assert_eq!(harness.text_of(".answer").as_deref(), Some("no host"));
}
