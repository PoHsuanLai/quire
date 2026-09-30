//! design/30 section 1.3, Collapse, on a real Blitz document: the content stands open or closed
//! on mount, a toggle moves its height and opacity over `--t-move`, a toggle mid-move reverses
//! from where it is, and under Reduced it is instant. Once settled nothing asks for a frame.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use ds_motion::use_collapse::use_collapse;
use ds_style::tokens::timing::DurationToken;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Hidden);

fn stage(motion: Motion) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Window,
            Body {}
        }
    }
}

#[component]
fn Body() -> Element {
    let collapse = use_collapse(SHOWN());
    rsx! { div { id: "body", style: collapse.style(Px(80.0)), "content" } }
}

#[allow(non_snake_case)]
fn Standard() -> Element {
    stage(Motion::Standard)
}

#[allow(non_snake_case)]
fn Reduced() -> Element {
    stage(Motion::Reduced)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn style(harness: &Harness) -> String {
    harness.attr("#body", "style").unwrap_or_default()
}

fn height(harness: &Harness) -> f32 {
    style(harness)
        .strip_prefix("height:")
        .and_then(|rest| rest.split("px").next())
        .and_then(|px| px.parse().ok())
        .unwrap_or(80.0)
}

fn set(harness: &mut Harness, shown: Shown) {
    harness.within(|| *SHOWN.write() = shown);
}

#[test]
fn it_stands_closed_on_mount_and_opens_over_one_move() {
    let mut harness = virtual_harness(Standard);
    harness.advance(Duration::from_millis(50));
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(20));
    assert_eq!(height(&harness), 0.0, "closed: {}", style(&harness));
    set(&mut harness, Shown::Visible);
    harness.advance(Duration::from_millis(60));
    let early = height(&harness);
    assert!(early > 0.0 && early < 40.0, "opening: {}", style(&harness));
    harness
        .advance(DurationToken::Move.duration(MotionLevel::Standard) + Duration::from_millis(60));
    assert_eq!(style(&harness), "", "open: natural size");
}

#[test]
fn a_toggle_mid_move_reverses_from_where_it_is() {
    let mut harness = virtual_harness(Standard);
    harness.advance(Duration::from_millis(50));
    set(&mut harness, Shown::Visible);
    harness.advance(Duration::from_millis(120));
    let middle = height(&harness);
    assert!(middle > 0.0 && middle < 80.0, "{middle}");
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(16));
    let after = height(&harness);
    assert!(
        (after - middle).abs() < 30.0,
        "no jump: {middle} then {after}"
    );
    harness.advance(Duration::from_millis(400));
    assert_eq!(height(&harness), 0.0);
}

#[test]
fn under_reduced_it_is_instant() {
    let mut harness = virtual_harness(Reduced);
    harness.advance(Duration::from_millis(50));
    set(&mut harness, Shown::Visible);
    harness.advance(Duration::from_millis(20));
    assert_eq!(style(&harness), "");
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(20));
    assert_eq!(height(&harness), 0.0);
}
