//! Stepper on a real Blitz document (design/30 section 2.1): a press steps once, held past the
//! repeat delay it repeats at the repeat pitch until it is let go, the keys step, and the ends
//! of the range hold.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Readout, ShortcutKey, StepRange, Stepper};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut value = use_signal(|| 5i32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            Stepper { label: "Copies", value: value(), range: StepRange::new(0, 100, 1), readout: Readout::Bare, onchange: move |next| value.set(next) }
            p { class: "value", "{value()}" }
        }
    }
}

fn value(harness: &Harness) -> i32 {
    harness
        .text_of(".value")
        .and_then(|text| text.parse().ok())
        .unwrap_or(-1)
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    harness
}

#[test]
fn a_press_steps_once_and_the_lower_half_steps_down() {
    let mut harness = harness();
    let up = harness.centre(".ds-stepper-up").expect("up half");
    harness.send(Input::click(up));
    assert_eq!(value(&harness), 6);
    let down = harness.centre(".ds-stepper-down").expect("down half");
    harness.send(Input::click(down));
    harness.send(Input::click(down));
    assert_eq!(value(&harness), 4);
}

#[test]
fn a_held_press_repeats_after_the_delay_and_stops_when_let_go() {
    let mut harness = harness();
    let up = harness.centre(".ds-stepper-up").expect("up half");
    harness.send(Input::pointer_down(up));
    assert_eq!(value(&harness), 6, "the press itself steps once");
    harness.advance(Duration::from_millis(400));
    assert_eq!(value(&harness), 6, "nothing repeats before the delay");
    harness.advance(Duration::from_millis(200));
    let after_delay = value(&harness);
    assert!(after_delay > 6, "repeating began: {after_delay}");
    harness.advance(Duration::from_millis(700));
    let later = value(&harness);
    assert!(
        later >= after_delay + 8,
        "about ten more steps in 700 ms: {after_delay} then {later}"
    );
    harness.send(Input::pointer_up(up));
    harness.advance(Duration::from_millis(500));
    assert_eq!(value(&harness), later, "released: it stops");
}

#[test]
fn the_arrow_keys_step_the_focused_pair() {
    let mut harness = harness();
    harness.send(Input::key(ShortcutKey::Tab));
    harness.send(Input::key(ShortcutKey::Up));
    assert_eq!(value(&harness), 6);
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Down));
    assert_eq!(value(&harness), 4);
    harness.send(Input::key(ShortcutKey::End));
    assert_eq!(value(&harness), 100);
    harness.send(Input::key(ShortcutKey::Up));
    assert_eq!(value(&harness), 100, "the top holds");
}
