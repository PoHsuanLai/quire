//! design/26 D1 on a real Blitz document: the battery status glyph (G8-G10). The fill sweeps
//! from where it is only when its drawn step changes, the bolt and the plug grow in, the low
//! tone is a colour claim, an Appear sweeps from empty, and Reduced jumps; each ends at 0 frames.

use dioxus::prelude::*;
use ds::detail::FirstShow;
use ds::{
    Appearance, BatteryGlyph, BatteryPower, BatteryState, Ds, Fraction, LowAt, Material, Motion,
};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 120,
    height: 80,
    scale_percent: 100,
};

fn battery(level: u16, power: BatteryPower) -> BatteryState {
    BatteryState {
        level: Fraction(level),
        power,
        low_at: LowAt::default(),
    }
}

static STATE: GlobalSignal<BatteryState> = Signal::global(|| battery(800, BatteryPower::Battery));
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { id: "battery", BatteryGlyph { state: STATE() } }
        }
    }
}

fn set(harness: &mut Harness, state: BatteryState) {
    harness.within(|| *STATE.write() = state);
}

/// The fill's drawn width on the 24 grid, 0 when there is none.
fn width(harness: &Harness) -> f32 {
    harness
        .attr("#battery [*|data-part=fill] rect", "width")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.0)
}

fn attr(harness: &Harness, part: &str, name: &str) -> Option<String> {
    harness.attr(&format!("#battery [*|data-part={part}]"), name)
}

#[test]
fn the_fill_sweeps_to_a_new_step_and_ignores_a_finer_change() {
    let mut harness = Harness::new(Page, VIEW);
    let full = width(&harness);
    // 80 % is step 18 of 22: 9 of the fill's 11 units.
    assert!((full - 9.0).abs() < 0.1, "80 % drew {full} units");
    assert_settles_to_zero_frames(&mut harness);
    // 80.0 then 79.8 % is the same step: nothing moves (R2).
    set(&mut harness, battery(798, BatteryPower::Battery));
    harness.advance(Duration::from_millis(30));
    let wakes = harness.wakes();
    harness.advance(Duration::from_millis(500));
    assert_eq!(harness.wakes(), wakes, "a finer change moved the fill");
    set(&mut harness, battery(300, BatteryPower::Battery));
    // 30 % is step 7: 3.5 units, reached through the widths between.
    settle_until(&mut harness, |h| (4.0..8.5).contains(&width(h)));
    settle_until(&mut harness, |h| (width(h) - 3.5).abs() < 0.1);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn plugging_in_grows_the_bolt_and_a_held_charge_shows_the_plug() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(
        attr(&harness, "bolt", "data-show").as_deref(),
        Some("hidden")
    );
    set(&mut harness, battery(800, BatteryPower::Charging));
    settle_until(&mut harness, |h| {
        attr(h, "bolt", "data-show").as_deref() == Some("lit")
    });
    assert_eq!(
        attr(&harness, "fill", "data-under").as_deref(),
        Some("mark")
    );
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, battery(1000, BatteryPower::Held));
    settle_until(&mut harness, |h| {
        attr(h, "plug", "data-show").as_deref() == Some("lit")
    });
    assert_eq!(
        attr(&harness, "bolt", "data-show").as_deref(),
        Some("hidden")
    );
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, battery(1000, BatteryPower::Battery));
    settle_until(&mut harness, |h| {
        attr(h, "plug", "data-show").as_deref() == Some("hidden")
    });
    assert_eq!(
        attr(&harness, "fill", "data-under").as_deref(),
        Some("none")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn the_fill_turns_low_at_the_threshold_but_never_while_charging() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(
        attr(&harness, "fill", "data-tone").as_deref(),
        Some("normal")
    );
    set(&mut harness, battery(150, BatteryPower::Battery));
    settle_until(&mut harness, |h| {
        attr(h, "fill", "data-tone").as_deref() == Some("low")
    });
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, battery(150, BatteryPower::Charging));
    settle_until(&mut harness, |h| {
        attr(h, "fill", "data-tone").as_deref() == Some("normal")
    });
    assert_settles_to_zero_frames(&mut harness);
}

/// A surface just opened: the glyph's first frame is an Appear.
#[allow(non_snake_case)]
fn Opened() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "battery",
                BatteryGlyph { state: battery(930, BatteryPower::Battery), first: FirstShow::Animate }
            }
        }
    }
}

#[test]
fn a_surface_just_opened_sweeps_the_fill_in_from_empty() {
    let mut harness = Harness::new(Opened, VIEW);
    settle_until(&mut harness, |h| width(h) > 0.5 && width(h) < 9.0);
    settle_until(&mut harness, |h| (width(h) - 10.0).abs() < 0.6);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_jumps_the_fill() {
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, battery(300, BatteryPower::Battery));
    harness.advance(Duration::from_millis(40));
    assert!((width(&harness) - 3.5).abs() < 0.1, "{}", width(&harness));
    assert_settles_to_zero_frames(&mut harness);
}
