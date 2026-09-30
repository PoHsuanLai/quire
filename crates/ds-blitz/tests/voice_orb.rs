//! The voice orb on a real Blitz document: its conic glows, dot grid and round mask paint (read
//! back from the pixels), it turns only while it is active, holds where it stands when it stops,
//! stays still under Reduced motion, and an idle orb wakes the document for nothing.

use dioxus::prelude::*;
use ds::{Activity, Appearance, Ds, Material, Motion, Px, VoiceOrb};
use ds_harness::harness::assert_settles_to_zero_frames;
use ds_harness::{Backdrop, Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 240,
    scale_percent: 100,
};

static ACTIVITY: GlobalSignal<Activity> = Signal::global(|| Activity::Inactive);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

/// A 192 px orb (a 20 s turn) on the window ground, its activity and motion set from outside.
#[allow(non_snake_case)]
fn Stage() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { style: "position:absolute; left:24px; top:24px;",
                VoiceOrb { size: Px(192.0), activity: ACTIVITY() }
            }
        }
    }
}

/// The orb at rest on the virtual clock: inactive, standard motion.
fn virtual_harness() -> Harness {
    Harness::with_config(Stage, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// Set the orb's activity from outside, as its owner would.
fn set(harness: &mut Harness, activity: Activity) {
    harness.within(|| *ACTIVITY.write() = activity);
}

/// The angle the orb is drawn at, in degrees, from its inline `--orb-turn`.
fn degrees(harness: &Harness) -> f32 {
    let style = harness
        .attr(".ds-voice-orb", "style")
        .expect("the orb is in the document");
    let rest = style
        .split("--orb-turn:")
        .nth(1)
        .unwrap_or_else(|| panic!("no --orb-turn in {style}"));
    rest.split("deg")
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| panic!("no angle in {style}"))
}

fn near(got: f32, want: f32) -> bool {
    (got - want).abs() < 2.0
}

#[test]
fn an_active_orb_turns_a_quarter_in_a_quarter_of_its_period() {
    let mut harness = virtual_harness();
    set(&mut harness, Activity::Active);
    let before = degrees(&harness);
    harness.advance(Duration::from_secs(5));
    let after = degrees(&harness);
    assert!(near(before, 0.0), "{before}");
    assert!(near(after, 90.0), "{after}");
}

#[test]
fn an_inactive_orb_stays_put_and_wakes_nothing() {
    let mut harness = virtual_harness();
    let woke = harness.wakes();
    harness.advance(Duration::from_secs(5));
    assert!(near(degrees(&harness), 0.0), "{}", degrees(&harness));
    assert_eq!(harness.wakes(), woke);
    assert!(!harness.is_animating());
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn an_orb_that_goes_inactive_holds_where_it_stands_and_stops_its_timer() {
    let mut harness = virtual_harness();
    set(&mut harness, Activity::Active);
    harness.advance(Duration::from_secs(5));
    set(&mut harness, Activity::Inactive);
    harness.advance(Duration::from_millis(100));
    let held = degrees(&harness);
    let woke = harness.wakes();
    harness.advance(Duration::from_secs(5));
    assert!(near(held, 90.0), "{held}");
    assert!(near(degrees(&harness), held), "{}", degrees(&harness));
    assert_eq!(harness.wakes(), woke, "an idle orb asked for frames");
}

#[test]
fn an_orb_that_wakes_again_carries_on_from_where_it_held() {
    let mut harness = virtual_harness();
    set(&mut harness, Activity::Active);
    harness.advance(Duration::from_secs(5));
    set(&mut harness, Activity::Inactive);
    harness.advance(Duration::from_secs(3));
    set(&mut harness, Activity::Active);
    harness.advance(Duration::from_secs(5));
    // A quarter turn, a rest, another quarter turn.
    assert!(near(degrees(&harness), 180.0), "{}", degrees(&harness));
}

#[test]
fn a_reduced_orb_does_not_turn() {
    let mut harness = virtual_harness();
    harness.within(|| *MOTION.write() = Motion::Reduced);
    set(&mut harness, Activity::Active);
    harness.advance(Duration::from_secs(5));
    assert!(near(degrees(&harness), 0.0), "{}", degrees(&harness));
}

/// The orb's own square, in the frame's pixels.
fn orb_pixels(harness: &mut Harness) -> (image::RgbaImage, u32, u32) {
    let frame = harness.render_over(Backdrop::Clear).expect("paints");
    let at = harness.rect(".ds-voice-orb").expect("the orb is laid out");
    (frame, at.origin.x.0 as u32, at.origin.y.0 as u32)
}

#[test]
fn the_orb_paints_a_round_field_of_colour() {
    let mut harness = virtual_harness();
    set(&mut harness, Activity::Active);
    let (frame, x0, y0) = orb_pixels(&mut harness);
    let alpha = |dx: u32, dy: u32| frame.get_pixel(x0 + dx, y0 + dy)[3];
    // The four corners of its square are outside the circle: nothing is painted there.
    for (dx, dy) in [(1, 1), (190, 1), (1, 190), (190, 190)] {
        assert_eq!(alpha(dx, dy), 0, "corner ({dx}, {dy}) is painted");
    }
    // The middle is painted, and a great part of the disc is: the glows and the ring.
    assert!(alpha(96, 96) > 0, "the middle is empty");
    let painted = (0..192)
        .flat_map(|dy| (0..192).map(move |dx| (dx, dy)))
        .filter(|(dx, dy)| alpha(*dx, *dy) > 0)
        .count();
    let disc = (std::f32::consts::PI * 96.0 * 96.0) as usize;
    assert!(
        painted * 100 > disc * 60,
        "{painted} of {disc} pixels painted"
    );
    assert!(
        painted <= disc + 2 * 192,
        "{painted} pixels leak past the circle"
    );
}

#[test]
fn the_glows_change_what_is_painted_as_the_orb_turns() {
    let mut harness = virtual_harness();
    set(&mut harness, Activity::Active);
    let (first, x0, y0) = orb_pixels(&mut harness);
    harness.advance(Duration::from_secs(5));
    let (later, _, _) = orb_pixels(&mut harness);
    let differing = (0..192)
        .flat_map(|dy| (0..192).map(move |dx| (x0 + dx, y0 + dy)))
        .filter(|(x, y)| first.get_pixel(*x, *y) != later.get_pixel(*x, *y))
        .count();
    assert!(
        differing > 500,
        "only {differing} pixels changed in a quarter turn"
    );
}
