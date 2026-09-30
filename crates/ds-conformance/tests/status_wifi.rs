//! design/26 on a real Blitz document: the Wi-Fi status glyph through every moment it plays, each
//! ending at 0 frames (R3), and under Reduced motion (R7).

use dioxus::prelude::*;
use ds::components::content::status::wifi_state::{WifiBars, WifiReach, WifiState};
use ds::detail::EventStamp;
use ds::prelude::*;
use ds_harness::harness::{assert_settles_to_zero_frames, settle_until};
use ds_harness::{ClassPresence, Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

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
fn joining_searches_at_once_and_a_join_lands_on_the_real_bars() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    set(&mut harness, WifiState::Idle);
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, WifiState::Joining(EventStamp(1)));
    harness.advance(Duration::from_millis(0));
    assert_eq!(lit(&harness), 1, "no grace: the loop shows at once");
    // One layer at a time, a step every --t-spin-step: the lit layer moves.
    let first = ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(&harness, part));
    harness.advance(Duration::from_millis(83));
    assert_eq!(lit(&harness), 1);
    assert_ne!(
        ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(&harness, part)),
        first
    );
    // Ten seconds in it is still searching: there is no cap.
    harness.advance(Duration::from_secs(10));
    assert_eq!(lit(&harness), 1);
    set(
        &mut harness,
        WifiState::Joined {
            bars: WifiBars::Two,
            reach: WifiReach::Internet,
        },
    );
    // The join lands on the real bars, at once: no fill.
    settle_until(&mut harness, |h| lit(h) == 3);
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(show(&harness, "arc-3").as_deref(), Some("faint"));
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
fn a_failed_join_leaves_the_fan_faint_and_shakes_nothing() {
    let mut harness = Harness::new(Page, VIEW);
    set(&mut harness, WifiState::Failed(EventStamp(1)));
    harness.advance(Duration::from_millis(60));
    assert_eq!(
        harness.has_class("#wifi .ds-status-glyph", "a-shake-x"),
        ClassPresence::Absent
    );
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(lit(&harness), 0, "the fan is faint");
}

#[test]
fn the_radio_off_draws_the_slash_on_and_back_off() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
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
fn reduced_keeps_the_search_turning_and_snaps_the_slash() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, WifiState::Joining(EventStamp(3)));
    harness.advance(Duration::from_millis(0));
    let first = ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(&harness, part));
    harness.advance(Duration::from_millis(83));
    assert_ne!(
        ["dot", "arc-1", "arc-2", "arc-3"].map(|part| show(&harness, part)),
        first,
        "the search keeps turning under Reduced"
    );
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
