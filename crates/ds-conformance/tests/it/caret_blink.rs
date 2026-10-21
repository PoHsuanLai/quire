//! A focused `TextField`'s caret blinks on a real Blitz document: 500 ms shown, 500 ms hidden
//! (`data-blink=off` on the wrap), shown again at once by a key, held still after ten seconds
//! with nothing typed (so the field goes back to 0 frames), and never under Reduced motion.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::harness::assert_settles_to_zero_frames;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 200,
    scale_percent: 100,
};

const WRAP: &str = "#a .ds-input-wrap";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn stage(motion: Motion) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Sheet,
            div { id: "a", style: "display:flex; width:300px; padding:12px",
                TextField { label: "A", value: "", oninput: |_| {} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Standard() -> Element {
    stage(Motion::Standard)
}

#[allow(non_snake_case)]
fn Reduced() -> Element {
    stage(Motion::Reduced)
}

fn focused(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    let field = harness.centre("#a input").expect("the field");
    harness.send(Input::click(field));
    harness
}

fn blink(harness: &Harness) -> Option<String> {
    harness.attr(WRAP, "data-blink")
}

#[test]
fn the_caret_blinks_shows_again_at_a_key_and_holds_still_after_ten_idle_seconds() {
    let mut harness = focused(Standard);
    harness.advance(ms(450));
    assert_eq!(blink(&harness), None, "shown for the first half second");
    harness.advance(ms(100));
    assert_eq!(
        blink(&harness).as_deref(),
        Some("off"),
        "hidden for the next"
    );
    harness.advance(ms(500));
    assert_eq!(blink(&harness), None, "shown again");

    harness.advance(ms(500));
    assert_eq!(blink(&harness).as_deref(), Some("off"));
    harness.send(Input::key(ShortcutKey::Char('x')));
    harness.advance(ms(10));
    assert_eq!(blink(&harness), None, "a key shows the caret at once");
    harness.advance(ms(400));
    assert_eq!(
        blink(&harness),
        None,
        "and the blink starts over from the key"
    );

    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(
        blink(&harness),
        None,
        "after ten idle seconds it holds still, shown"
    );
}

#[test]
fn under_reduced_motion_the_caret_never_blinks() {
    let mut harness = focused(Reduced);
    for _ in 0..4 {
        harness.advance(ms(300));
        assert_eq!(blink(&harness), None);
    }
    assert_settles_to_zero_frames(&mut harness);
}
