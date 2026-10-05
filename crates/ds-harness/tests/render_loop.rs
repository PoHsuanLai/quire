//! A render loop in the app under test fails the harness loudly instead of being cut off after
//! the round cap and carried on from.

use dioxus::prelude::*;
use ds_harness::{Driver, Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 100,
    scale_percent: 100,
};

/// An effect that reads and writes the same signal re-runs, and re-renders, forever.
#[allow(non_snake_case)]
fn Spinner() -> Element {
    let mut turns = use_signal(|| 0_u64);
    use_effect(move || turns.set(turns() + 1));
    rsx! { div { id: "spinner", "turns {turns}" } }
}

/// Settles after a few renders: more than one round, far below the cap.
#[allow(non_snake_case)]
fn Settler() -> Element {
    let mut turns = use_signal(|| 0_u64);
    use_effect(move || {
        if turns() < 5 {
            turns.set(turns() + 1);
        }
    });
    rsx! { div { id: "settler", "turns {turns}" } }
}

#[test]
#[should_panic(expected = "render loop")]
fn a_component_that_re_renders_forever_fails_the_harness() {
    let _ = Harness::new(Spinner, VIEW);
}

#[test]
fn a_loop_names_the_document_it_was_stuck_in() {
    let caught = std::panic::catch_unwind(|| {
        let _ = Harness::new(Spinner, VIEW);
    })
    .expect_err("the loop must panic");
    let message = caught
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| caught.downcast_ref::<&str>().map(|text| (*text).to_owned()))
        .expect("a text panic message");
    assert!(message.contains("render loop"), "{message}");
    assert!(message.contains("id=\"spinner\""), "{message}");
}

#[test]
fn a_document_that_settles_after_a_few_rounds_is_not_a_loop() {
    let mut harness = Harness::new(Settler, VIEW);
    harness.advance(std::time::Duration::from_millis(10));
    assert!(harness.html().contains("turns 5"), "{}", harness.html());
}
