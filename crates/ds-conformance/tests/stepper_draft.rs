//! A number half typed into a Stepper's field does not survive a step: the arrow keys and the
//! halves replace it with the number they land on, rather than leaving it to jump the value on
//! blur.

use dioxus::prelude::*;
use ds::components::fields::stepper::model::{Readout, StepRange};
use ds::components::fields::stepper::view::Stepper;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut value = use_signal(|| 3i32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            Stepper { label: "Copies", value: value(), range: StepRange::new(0, 100, 1), readout: Readout::Field, onchange: move |next| value.set(next) }
            p { class: "value", "{value()}" }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    harness
}

fn shown(harness: &Harness) -> Option<String> {
    harness.attr(".ds-stepper input", "value")
}

/// Type `digits` into the field, leaving them uncommitted.
fn type_into_field(harness: &mut Harness, digits: &str) {
    let field = harness.centre(".ds-stepper input").expect("the field");
    harness.send(Input::click(field));
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('a')));
    for digit in digits.chars() {
        harness.send(Input::key(ShortcutKey::Char(digit)));
    }
    harness.advance(Duration::from_millis(30));
}

#[test]
fn an_arrow_key_in_the_field_steps_from_the_value_and_drops_the_half_typed_number() {
    let mut harness = harness();
    type_into_field(&mut harness, "5");
    assert_eq!(
        shown(&harness).as_deref(),
        Some("5"),
        "typed, not committed"
    );
    harness.send(Input::key(ShortcutKey::Up));
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.text_of(".value").as_deref(), Some("4"));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("4"),
        "the field shows the step"
    );
}

#[test]
fn a_press_on_a_half_drops_the_half_typed_number_too() {
    let mut harness = harness();
    type_into_field(&mut harness, "5");
    let down = harness.centre(".ds-stepper-down").expect("down half");
    harness.send(Input::click(down));
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.text_of(".value").as_deref(), Some("2"));
    assert_eq!(shown(&harness).as_deref(), Some("2"));
}
