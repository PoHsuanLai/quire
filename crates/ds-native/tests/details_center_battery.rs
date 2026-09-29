//! design/26 on a real Blitz document, on the virtual clock: the Battery module's device ring
//! sweeping in from empty over `--t-sweep` on a center just opened, its percentage counting in
//! step and landing on the true value, a later level sweeping from where it is, the bolt waiting
//! for the sweep; and the keyboard-brightness level's glyph. Each ends at 0 frames;
//! Reduced shows the level at once.

use dioxus::prelude::*;
use ds::detail::FirstShow;
use ds::{
    Appearance, DeviceBattery, Ds, Fraction, LevelControl, LevelGlyph, Material, Motion, RingMark,
};
use ds_native::harness::assert_settles_to_zero_frames;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 200,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

static LEVEL: GlobalSignal<Fraction> = Signal::global(|| Fraction(930));
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Module() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { id: "mac", DeviceBattery { level: LEVEL(), mark: RingMark::Charging, label: "This computer", first: FirstShow::Animate } }
            div { id: "keys", LevelControl { label: "Keyboard Brightness".to_owned(), value: Fraction(500), glyph: LevelGlyph::KeyboardBrightness } }
        }
    }
}

fn figure(harness: &Harness) -> Option<u32> {
    harness
        .text_of("#mac .ds-battery-figure")
        .and_then(|text| text.trim_end_matches('%').parse().ok())
}

/// The level's arc, absent while it is empty.
fn arc(harness: &Harness) -> Option<String> {
    harness.attr("#mac .ds-battery-arc path", "d")
}

#[test]
fn the_ring_sweeps_in_with_its_percentage_counting_in_step() {
    let mut harness = virtual_harness(Module);
    assert_eq!(figure(&harness), Some(0), "the count starts at zero");
    assert_eq!(arc(&harness), None, "the arc starts empty");
    assert_eq!(
        harness.attr("#mac .ds-battery", "aria-valuenow").as_deref(),
        Some("93"),
        "the true level is stated from the first frame (R8)"
    );
    assert_eq!(
        harness
            .attr("#mac .ds-battery-bolt", "data-show")
            .as_deref(),
        Some("off"),
        "the bolt waits for the sweep"
    );
    harness.advance(ms(200));
    let midway = figure(&harness).unwrap_or(0);
    assert!(midway > 0 && midway < 93, "midway: {midway}");
    assert!(arc(&harness).is_some(), "the arc is under way");
    harness.advance(ms(500));
    assert_eq!(
        figure(&harness),
        Some(93),
        "it lands on the true value with the sweep"
    );
    // The sweep's last frame (the next 16 ms frame past --t-sweep) lets the bolt in.
    harness.advance(ms(20));
    assert_eq!(
        harness
            .attr("#mac .ds-battery-bolt", "data-show")
            .as_deref(),
        Some("on")
    );
    assert_settles_to_zero_frames(&mut harness);
    // A later level sweeps from where it is over --t-quick, counting down.
    harness.within(|| *LEVEL.write() = Fraction(600));
    harness.advance(ms(0));
    harness.advance(ms(80));
    let between = figure(&harness).unwrap_or(0);
    assert!(between < 93 && between > 60, "from where it was: {between}");
    harness.advance(ms(400));
    assert_eq!(figure(&harness), Some(60));
    assert_settles_to_zero_frames(&mut harness);
    // A one-point step prints at once (R12).
    harness.within(|| *LEVEL.write() = Fraction(590));
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(59));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_shows_the_level_and_the_bolt_at_once() {
    let mut harness = virtual_harness(Module);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.within(|| *LEVEL.write() = Fraction(410));
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(41));
    assert_eq!(
        harness
            .attr("#mac .ds-battery-bolt", "data-show")
            .as_deref(),
        Some("on")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn the_keyboard_light_draws_a_keyboard_under_a_sun_whose_rays_follow_the_level() {
    let mut harness = virtual_harness(Module);
    for part in ["key-body", "key-core", "key-rays"] {
        assert!(
            harness.count(&format!("#keys [*|data-part={part}]")) >= 1,
            "{part}: {}",
            harness.html()
        );
    }
    assert_eq!(
        harness.count("#keys [*|data-part=rays]"),
        0,
        "not the display's sun"
    );
    assert_settles_to_zero_frames(&mut harness);
}
