//! `Capsule` on a real Blitz document: it stands at the bottom centre of what it floats over,
//! each button reports its value, the pointer entering and leaving it is reported (so an owner
//! can keep it up), and a hide fades it out, calls `on_hidden` once and takes the pointer away.

use dioxus::prelude::*;
use ds::components::chrome::capsule::model::CapsuleSlot;
use ds::components::chrome::capsule::view::Capsule;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// The `--t-quick` fade, which the capsule's presence settles by.
fn fade() -> Duration {
    settle(Anim::PaletteFade, MotionLevel::Standard)
}

/// A capsule over a 640 x 400 stage; `.log` counts what happened, and Escape-free: the stage
/// click toggles `shown`.
#[allow(non_snake_case)]
fn Stage() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut shown = use_signal(|| Shown::Visible);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "stage", style: "position:relative; width:640px; height:400px",
                button { class: "toggle", onclick: move |_| shown.set(shown().flipped()), "toggle" }
                Capsule::<u8> {
                    label: "Controls",
                    slots: vec![
                        CapsuleSlot::button(1, "Zoom out", Icon::Minus),
                        CapsuleSlot::Readout("100%".to_owned()),
                        CapsuleSlot::button(2, "Zoom in", Icon::Plus),
                    ],
                    shown: shown(),
                    on_hidden: move |()| log.with_mut(|log| log.push("hidden".to_owned())),
                    onpick: move |value| log.with_mut(|log| log.push(format!("pick {value}"))),
                    onpointerenter: move |()| log.with_mut(|log| log.push("enter".to_owned())),
                    onpointerleave: move |()| log.with_mut(|log| log.push("leave".to_owned())),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Stage, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    harness
}

#[test]
fn it_stands_at_the_bottom_centre_of_what_it_floats_over() {
    let harness = harness();
    let stage = harness.rect(".stage").expect("the stage");
    let capsule = harness.rect(".ds-capsule").expect("the capsule");
    let middle = capsule.origin.x.0 + capsule.size.width.0 / 2.0;
    assert!(
        (middle - (stage.origin.x.0 + stage.size.width.0 / 2.0)).abs() < 2.0,
        "centred: {capsule:?} in {stage:?}"
    );
    let gap = stage.origin.y.0 + stage.size.height.0 - (capsule.origin.y.0 + capsule.size.height.0);
    assert!(
        (0.0..=40.0).contains(&gap),
        "it sits just above the bottom edge: {gap}"
    );
}

#[test]
fn a_button_reports_its_value_and_the_pointer_over_it_is_reported() {
    let mut harness = harness();
    let zoom_in = harness
        .centre(".ds-capsule .ds-button:nth-of-type(2)")
        .expect("the second button");
    harness.send(Input::pointer_move(zoom_in));
    harness.advance(Duration::from_millis(20));
    assert_eq!(harness.text_of(".log").as_deref(), Some("enter"));
    harness.send(Input::click(zoom_in));
    harness.advance(Duration::from_millis(20));
    assert_eq!(harness.text_of(".log").as_deref(), Some("enter,pick 2"));
    let away = Point {
        x: Px(10.0),
        y: Px(10.0),
    };
    harness.send(Input::pointer_move(away));
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("enter,pick 2,leave")
    );
}

#[test]
fn a_hide_fades_out_calls_on_hidden_once_and_lets_the_pointer_through() {
    let mut harness = harness();
    assert_eq!(
        harness.attr(".ds-capsule", "data-shown").as_deref(),
        Some("visible")
    );
    let toggle = harness.centre(".toggle").expect("the toggle");
    harness.send(Input::click(toggle));
    harness.advance(fade() / 2);
    assert_eq!(
        harness.attr(".ds-capsule", "data-presence").as_deref(),
        Some("leaving"),
        "mid-fade it is still drawn, leaving"
    );
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some(""),
        "not hidden yet"
    );
    harness.advance(fade());
    assert_eq!(harness.text_of(".log").as_deref(), Some("hidden"));
    assert_eq!(
        harness.attr(".ds-capsule", "data-shown").as_deref(),
        Some("hidden")
    );
    assert_eq!(
        harness.rect(".ds-capsule").map(|rect| rect.size.width.0),
        Some(0.0),
        "hidden, nothing is laid out"
    );
}
