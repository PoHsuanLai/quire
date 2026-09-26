//! design/26 D1 on a real Blitz document: the Wi-Fi status glyph through every moment of its
//! table (G1-G5), each ending at 0 frames (R3), and under Reduced motion (R7).

use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{Appearance, Ds, Material, Motion, WifiBars, WifiGlyph, WifiReach, WifiState};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 120,
    height: 80,
    scale_percent: 100,
};

const THREE: WifiState = WifiState::Joined {
    bars: WifiBars::Three,
    reach: WifiReach::Internet,
};

static STATE: GlobalSignal<WifiState> = Signal::global(|| THREE);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { id: "wifi", WifiGlyph { state: STATE() } }
        }
    }
}

fn set(harness: &mut Harness, state: WifiState) {
    harness.within(|| *STATE.write() = state);
}

fn show(harness: &Harness, part: &str) -> Option<String> {
    harness.attr(&format!("#wifi [*|data-part={part}]"), "data-show")
}

fn lit(harness: &Harness) -> usize {
    ["dot", "arc-1", "arc-2", "arc-3"]
        .iter()
        .filter(|part| show(harness, part).as_deref() == Some("lit"))
        .count()
}

fn slash_offset(harness: &Harness) -> Option<f32> {
    harness
        .attr("#wifi [*|data-part=slash] path", "stroke-dashoffset")
        .and_then(|value| value.parse::<f32>().ok())
}

#[test]
fn at_rest_a_joined_glyph_is_whole_and_a_repeat_plays_nothing() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(lit(&harness), 4);
    assert_eq!(show(&harness, "badge").as_deref(), Some("hidden"));
    assert_eq!(harness.count("#wifi [*|data-part=slash]"), 0);
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, THREE);
    harness.advance(Duration::from_millis(30));
    let wakes = harness.wakes();
    harness.advance(Duration::from_millis(500));
    assert_eq!(harness.wakes(), wakes, "the same state started a timer");
    assert!(!harness.is_animating());
}

#[test]
fn joining_searches_after_its_grace_then_a_join_fills_once_to_the_real_bars() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, WifiState::Idle);
    assert_settles_to_zero_frames(&mut harness);
    let asked = Instant::now();
    set(&mut harness, WifiState::Joining(EventStamp(1)));
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        lit(&harness),
        0,
        "a join younger than PendingGrace shows no loop"
    );
    let searching = settle_until(&mut harness, |h| lit(h) == 1);
    assert!(
        searching - asked >= Duration::from_millis(400),
        "the loop ignored its grace"
    );
    // One layer at a time: the lit layer moves.
    let first = ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(&harness, part));
    settle_until(&mut harness, |h| {
        lit(h) == 1 && ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(h, part)) != first
    });
    set(
        &mut harness,
        WifiState::Joined {
            bars: WifiBars::Two,
            reach: WifiReach::Internet,
        },
    );
    // Settle(Fill): the dot alone first, then out to the second arc, and it stays there.
    settle_until(&mut harness, |h| {
        lit(h) == 1 && show(h, "dot").as_deref() == Some("lit")
    });
    settle_until(&mut harness, |h| lit(h) == 3);
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(show(&harness, "arc-3").as_deref(), Some("faint"));
    assert_eq!(lit(&harness), 3);
}

#[test]
fn a_join_that_outlives_the_cap_holds_its_still_frame_at_zero_frames() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, WifiState::Joining(EventStamp(7)));
    let cap = Duration::from_secs(10);
    let started = Instant::now();
    while started.elapsed() < cap {
        harness.advance(Duration::from_millis(250));
    }
    settle_until(&mut harness, |h| {
        h.attr("#wifi .ds-status-glyph", "data-pending").as_deref() == Some("still")
    });
    assert_eq!(
        lit(&harness),
        4,
        "the still frame is the whole glyph, dimmed"
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn bars_cross_fade_and_no_internet_grows_the_badge() {
    let mut harness = Harness::new(Page, VIEW);
    set(
        &mut harness,
        WifiState::Joined {
            bars: WifiBars::One,
            reach: WifiReach::Internet,
        },
    );
    settle_until(&mut harness, |h| lit(h) == 2);
    assert_settles_to_zero_frames(&mut harness);
    set(
        &mut harness,
        WifiState::Joined {
            bars: WifiBars::One,
            reach: WifiReach::NoInternet,
        },
    );
    settle_until(&mut harness, |h| show(h, "badge").as_deref() == Some("lit"));
    assert_eq!(lit(&harness), 2, "the bars stay true under the badge");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_failed_join_shakes_once_per_stamp() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, WifiState::Failed(EventStamp(1)));
    settle_until(&mut harness, |h| {
        h.has_class("#wifi .ds-status-glyph", "a-shake-x")
    });
    settle_until(&mut harness, |h| {
        !h.has_class("#wifi .ds-status-glyph", "a-shake-x")
    });
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(lit(&harness), 0, "after the shake the fan is faint");
    // The same failure again is no moment (R6).
    set(&mut harness, WifiState::Failed(EventStamp(1)));
    harness.advance(Duration::from_millis(60));
    assert!(!harness.has_class("#wifi .ds-status-glyph", "a-shake-x"));
    // A new one shakes the same way.
    set(&mut harness, WifiState::Failed(EventStamp(2)));
    settle_until(&mut harness, |h| {
        h.has_class("#wifi .ds-status-glyph", "a-shake-x")
    });
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn the_radio_off_draws_the_slash_on_and_back_off() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, WifiState::Off);
    settle_until(&mut harness, |h| slash_offset(h).is_some_and(|o| o > 0.0));
    settle_until(&mut harness, |h| slash_offset(h) == Some(0.0));
    assert_eq!(lit(&harness), 0);
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, WifiState::Idle);
    settle_until(&mut harness, |h| h.count("#wifi [*|data-part=slash]") == 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_holds_the_still_frame_snaps_the_slash_and_does_not_shake() {
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, WifiState::Joining(EventStamp(3)));
    settle_until(&mut harness, |h| {
        h.attr("#wifi .ds-status-glyph", "data-pending").as_deref() == Some("still")
    });
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, WifiState::Failed(EventStamp(3)));
    harness.advance(Duration::from_millis(60));
    assert!(!harness.has_class("#wifi .ds-status-glyph", "a-shake-x"));
    set(&mut harness, WifiState::Off);
    harness.advance(Duration::from_millis(40));
    assert_eq!(
        slash_offset(&harness),
        Some(0.0),
        "the slash drew on under Reduced"
    );
    set(&mut harness, THREE);
    harness.advance(Duration::from_millis(40));
    assert_eq!(lit(&harness), 4);
    assert_settles_to_zero_frames(&mut harness);
}
