//! A listener that moves its own content with the eased detents it hears (the PDF and text views)
//! shows the move in the frame that handed it the detents. The window steps scrolling ahead of
//! the frame's renders, so the listener's write is rendered, laid out and painted together;
//! stepped after them, the content trailed the scroll by a frame. The harness runs the same
//! order, and each `advance` of one frame here is one frame of the window.

use dioxus::prelude::*;
use ds::host::gesture::{Gesture, WheelDelivery, use_gestures_with};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const FRAME: Duration = Duration::from_millis(8);

thread_local! {
    /// The px of content motion the listener has been handed so far.
    static HEARD: RefCell<f32> = const { RefCell::new(0.0) };
}

/// A bar as tall as the detents the listener has heard: content the app moves itself.
#[allow(non_snake_case)]
fn Follower() -> Element {
    let mut moved = use_signal(|| 0.0_f32);
    use_gestures_with(WheelDelivery::Eased, move |gesture| {
        if let Gesture::Scroll { by, .. } = gesture {
            HEARD.with(|heard| *heard.borrow_mut() -= by.y.0);
            moved -= by.y.0;
        }
    });
    rsx! {
        div { id: "area", style: "width:300px; height:300px; margin:0",
            div { id: "bar", style: "width:20px; height:{moved}px; background:red" }
        }
    }
}

fn heard() -> f32 {
    HEARD.with(|heard| *heard.borrow())
}

#[test]
fn what_a_listener_heard_this_frame_is_painted_in_this_frame() {
    HEARD.with(|heard| *heard.borrow_mut() = 0.0);
    let mut harness = Harness::new(
        Follower,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let at = harness.centre("#area").expect("laid out");
    harness.send(Input::detents(at, 0.0, -1.0));
    let mut seen = Vec::new();
    for _ in 0..30 {
        harness.advance(FRAME);
        let painted = harness.rect("#bar").map_or(0.0, |rect| rect.size.height.0);
        seen.push((heard(), painted));
    }
    assert!(
        seen.iter().any(|(heard, _)| *heard > 0.0),
        "the listener heard the detent: {seen:?}"
    );
    for (n, (heard, painted)) in seen.iter().enumerate() {
        assert!(
            (heard - painted).abs() <= 1.0,
            "frame {n}: the listener has heard {heard} px but the frame paints {painted}: {seen:?}"
        );
    }
}
