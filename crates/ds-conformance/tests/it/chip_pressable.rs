//! A pressable `Chip`: its face is a real button with `aria-pressed`, answers a click, Return
//! and Space, and its remove button (on any variant, not Person alone) removes without pressing.

use dioxus::prelude::*;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::components::controls::chip_dot::ChipDot;
use ds::prelude::*;
use ds_harness::{Clock, FocusState, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

const FACE: &str = ".ds-chip-press";
const REMOVE: &str = ".ds-chip-remove";

#[allow(non_snake_case)]
fn App() -> Element {
    let mut presses = use_signal(|| 0u32);
    let mut removed = use_signal(|| 0u32);
    let pressed = if presses() % 2 == 1 {
        Check::On
    } else {
        Check::Off
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:40px; display:flex; gap:12px",
                Chip {
                    variant: ChipVariant::Neutral,
                    text: "Work",
                    dot: ChipDot::Colour("#3366cc".to_owned()),
                    pressed,
                    onpress: move |_| presses += 1,
                    onremove: move |()| removed += 1,
                }
            }
            p { class: "probe-presses", "{presses}" }
            p { class: "probe-removed", "{removed}" }
        }
    }
}

fn harness() -> Harness {
    Harness::new(App, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn count(harness: &Harness, probe: &str) -> String {
    harness.text_of(probe).unwrap_or_default()
}

#[test]
fn a_pressable_chip_is_a_button_with_its_pressed_state_and_a_dot() {
    let harness = harness();
    assert_eq!(harness.count("span.ds-chip > button.ds-chip-press"), 1);
    assert_eq!(harness.attr(FACE, "aria-pressed").as_deref(), Some("false"));
    assert_eq!(harness.count("i.ds-chip-dot"), 1, "{}", harness.html());
}

#[test]
fn a_click_presses_the_chip_and_its_state_follows() {
    let mut harness = harness();
    let at = harness.centre(FACE).expect("the face is on screen");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(50));
    assert_eq!(count(&harness, ".probe-presses"), "1");
    assert_eq!(harness.attr(FACE, "aria-pressed").as_deref(), Some("true"));
}

#[test]
fn return_and_space_press_the_focused_chip() {
    let mut harness = harness();
    harness.send(Input::key(ShortcutKey::Tab));
    assert_eq!(harness.focus_of(FACE), FocusState::Focused);
    harness.send(Input::key(ShortcutKey::Enter));
    assert_eq!(count(&harness, ".probe-presses"), "1");
    harness.send(Input::key(ShortcutKey::Space));
    assert_eq!(count(&harness, ".probe-presses"), "2");
}

#[test]
fn removing_a_chip_that_is_not_a_person_does_not_press_it() {
    let mut harness = harness();
    let at = harness.centre(REMOVE).expect("the remove button is shown");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(50));
    assert_eq!(count(&harness, ".probe-removed"), "1");
    assert_eq!(count(&harness, ".probe-presses"), "0");
}
