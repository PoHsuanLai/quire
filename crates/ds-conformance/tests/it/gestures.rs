//! Touchpad gestures and pointer capture for any element, on a real Blitz document: a pinch and
//! a scroll with its phase reach the component listening, in order; a wheel publishes a scroll;
//! and a press that captured the pointer keeps hearing moves and the release outside the element
//! (Blitz would deliver them only to what is under the pointer).

use dioxus::prelude::*;
use ds::host::captured::CapturedPointer;
use ds::host::gesture::{Gesture, GesturePhase, Magnification, ScrollSource, use_gestures};
use ds::host::pointer_capture::{PointerHold, use_pointer_capture};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Writes every gesture it hears into `.log`.
#[allow(non_snake_case)]
fn Listener() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    use_gestures(move |gesture| {
        let line = match gesture {
            Gesture::Pinch { phase, by, .. } => format!("pinch {phase:?} {}", by.0),
            Gesture::Scroll {
                phase, by, held, ..
            } => {
                let control = held.contains(Modifiers::CONTROL);
                format!("scroll {phase:?} {} {} control={control}", by.x.0, by.y.0)
            }
        };
        log.with_mut(|log| log.push(line));
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:400px; height:300px" }
            p { class: "log", {log().join("; ")} }
        }
    }
}

#[test]
fn a_pinch_and_a_scroll_arrive_in_order_with_their_phases() {
    let mut harness = Harness::new(
        Listener,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(50));
    let at = Point {
        x: Px(100.0),
        y: Px(80.0),
    };
    for (phase, by) in [
        (GesturePhase::Began, 0),
        (GesturePhase::Changed, 30),
        (GesturePhase::Ended, -10),
    ] {
        harness.send(Input::gesture(Gesture::Pinch {
            phase,
            by: Magnification(by),
            at,
        }));
    }
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("pinch Began 0; pinch Changed 30; pinch Ended -10")
    );
}

#[test]
fn a_wheel_is_published_as_a_scroll() {
    let mut harness = Harness::new(
        Listener,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(50));
    harness.send(Input::wheel(
        Point {
            x: Px(50.0),
            y: Px(50.0),
        },
        Px(0.0),
        Px(-24.0),
    ));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("scroll Changed 0 -24 control=false")
    );
}

#[test]
fn a_scroll_carries_the_modifiers_held_while_it_happened() {
    let mut harness = Harness::new(
        Listener,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(50));
    harness.send(Input::gesture(Gesture::Scroll {
        source: ScrollSource::Finger,
        phase: GesturePhase::Changed,
        by: Point {
            x: Px(0.0),
            y: Px(12.0),
        },
        at: Point {
            x: Px(50.0),
            y: Px(50.0),
        },
        held: Modifiers::CONTROL,
    }));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("scroll Changed 0 12 control=true")
    );
}

/// A 100 px square that follows the pointer once pressed; `.log` lists what it heard.
#[allow(non_snake_case)]
fn Handle() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let hold = use_pointer_capture(move |pointer: CapturedPointer| {
        log.with_mut(|log| {
            log.push(format!(
                "{:?} {} {}",
                pointer.phase, pointer.at.x.0, pointer.at.y.0
            ))
        });
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                class: "handle",
                style: "width:100px; height:100px",
                onmounted: move |event| hold.on_mounted(event),
                onpointerdown: move |_| {
                    let held = hold.begin();
                    log.with_mut(|log| log.push(match held {
                        PointerHold::Captured => "captured".to_owned(),
                        PointerHold::Local => "local".to_owned(),
                    }));
                },
            }
            p { class: "log", {log().join("; ")} }
        }
    }
}

#[test]
fn a_captured_pointer_is_followed_outside_the_element_until_the_release() {
    let mut harness = Harness::new(Handle, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    let inside = harness.centre(".handle").expect("the handle");
    harness.send(Input::pointer_down(inside));
    harness.advance(ms(20));
    let outside = Point {
        x: Px(300.0),
        y: Px(250.0),
    };
    harness.send(Input::pointer_move(outside));
    harness.send(Input::pointer_up(outside));
    harness.advance(ms(20));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("captured; Drag 300 250; Release 300 250")
    );
    // The release ended the capture: a later move goes nowhere.
    harness.send(Input::pointer_move(inside));
    harness.advance(ms(20));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("captured; Drag 300 250; Release 300 250")
    );
}
