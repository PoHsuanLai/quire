//! SegmentedControl's roving keys (design/30 section 1.4) on a real Blitz document: with a segment
//! focused, Right and Left move the choice one segment and stop at the ends, Home and End jump
//! to them.

use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use ds_style::tokens::control_size::ControlSize;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut value = use_signal(|| 1u8);
    let options = vec![
        (0, "Low".to_owned()),
        (1, "Mid".to_owned()),
        (2, "High".to_owned()),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            SegmentedControl::<u8> { label: "Level", choices: Choice::pairs(options), tracking: Tracking::SelectOne(value()), size: ControlSize::Regular, onchange: move |next| value.set(next) }
            p { class: "value", "{value()}" }
        }
    }
}

fn value(harness: &Harness) -> String {
    harness.text_of(".value").unwrap_or_default()
}

#[test]
fn arrows_move_the_choice_and_stop_at_the_ends() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    harness.send(Input::key(ShortcutKey::Tab));
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(value(&harness), "2");
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(value(&harness), "2", "the end stops");
    harness.send(Input::key(ShortcutKey::Left));
    assert_eq!(value(&harness), "1");
    harness.send(Input::key(ShortcutKey::Home));
    assert_eq!(value(&harness), "0");
    harness.send(Input::key(ShortcutKey::End));
    assert_eq!(value(&harness), "2");
}
