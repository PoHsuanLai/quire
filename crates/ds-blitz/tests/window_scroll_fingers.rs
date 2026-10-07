//! What a component that moves its own content hears of fingers on a touchpad and of a wheel
//! under Control (`ds::host::gesture`): the fingers' run with its source and phases, the glide
//! after a fast lift for a listener that asked for the ease, and a zoom wheel delivered whole.
//! The harness runs the window's scroll step on its virtual clock, as the window loop does.

use dioxus::prelude::*;
use ds::host::gesture::{
    Gesture, GesturePhase, ScrollSource, WheelDelivery, use_gestures, use_gestures_with,
};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Viewport};
use keyboard_types::Modifiers;
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

/// One scroll gesture a listener heard.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Heard {
    source: ScrollSource,
    phase: GesturePhase,
    y: f32,
    zoom: bool,
}

thread_local! {
    /// What each kind of listener heard, per thread: each test runs its harness on its own.
    static AS_RECEIVED: RefCell<Vec<Heard>> = const { RefCell::new(Vec::new()) };
    static EASED: RefCell<Vec<Heard>> = const { RefCell::new(Vec::new()) };
}

fn hear(log: &'static std::thread::LocalKey<RefCell<Vec<Heard>>>, gesture: Gesture) {
    if let Gesture::Scroll {
        source,
        phase,
        by,
        held,
        ..
    } = gesture
    {
        let zoom = held.contains(Modifiers::CONTROL);
        let heard = Heard {
            source,
            phase,
            y: by.y.0,
            zoom,
        };
        log.with(|log| log.borrow_mut().push(heard));
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    use_gestures(|gesture| hear(&AS_RECEIVED, gesture));
    use_gestures_with(WheelDelivery::Eased, |gesture| hear(&EASED, gesture));
    rsx! { div { style: "width:400px; height:400px" } }
}

fn harness() -> Harness {
    AS_RECEIVED.with(|log| log.borrow_mut().clear());
    EASED.with(|log| log.borrow_mut().clear());
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn heard(log: &'static std::thread::LocalKey<RefCell<Vec<Heard>>>) -> Vec<Heard> {
    log.with(|log| log.borrow().clone())
}

const AT: Point = Point {
    x: Px(100.0),
    y: Px(100.0),
};

const FRAME: Duration = Duration::from_millis(8);

/// Fingers moving `step` px down every frame for `moves` moves, then lifting with no last
/// motion.
fn flick(harness: &mut Harness, step: f32, moves: u32) {
    for n in 0..moves {
        let phase = match n {
            0 => GesturePhase::Began,
            _ => GesturePhase::Changed,
        };
        harness.send(Input::fingers(AT, Px(0.0), Px(step), phase));
        harness.advance(FRAME);
    }
    harness.send(Input::fingers(AT, Px(0.0), Px(0.0), GesturePhase::Ended));
}

fn phases(heard: &[Heard]) -> Vec<GesturePhase> {
    heard.iter().map(|h| h.phase).collect()
}

#[test]
fn fingers_reach_a_listener_as_received_with_their_source_and_phases() {
    let mut harness = harness();
    flick(&mut harness, 30.0, 4);
    let got = heard(&AS_RECEIVED);
    assert_eq!(
        phases(&got),
        [
            GesturePhase::Began,
            GesturePhase::Changed,
            GesturePhase::Changed,
            GesturePhase::Changed,
            GesturePhase::Ended
        ]
    );
    assert!(
        got.iter().all(|h| h.source == ScrollSource::Finger),
        "{got:?}"
    );
    assert_eq!(got[0].y, 30.0, "the fingers' px, unchanged");
}

#[test]
fn an_eased_listener_hears_the_glide_after_a_fast_lift_and_ends_when_it_stops() {
    let mut harness = harness();
    flick(&mut harness, 30.0, 6);
    let at_lift = heard(&EASED);
    assert_eq!(
        phases(&at_lift),
        [
            GesturePhase::Began,
            GesturePhase::Changed,
            GesturePhase::Changed,
            GesturePhase::Changed,
            GesturePhase::Changed,
            GesturePhase::Changed
        ],
        "the lift of a fast flick is not an end yet"
    );
    assert_eq!(
        heard(&AS_RECEIVED).last().map(|h| h.phase),
        Some(GesturePhase::Ended),
        "an as-received listener hears the lift at once"
    );

    let before = at_lift.len();
    for _ in 0..400 {
        harness.advance(FRAME);
    }
    let all = heard(&EASED);
    let glide = &all[before..];
    let (last, moving) = glide.split_last().expect("the glide spoke");
    assert_eq!(last.phase, GesturePhase::Ended, "{glide:?}");
    assert!(
        moving.iter().all(|h| h.phase == GesturePhase::Changed),
        "{glide:?}"
    );
    assert!(
        moving.len() > 10,
        "frame by frame, not a jump: {}",
        moving.len()
    );
    let distance: f32 = moving.iter().map(|h| h.y).sum();
    assert!(
        distance > 100.0,
        "a flick at 30 px a frame coasts on: {distance}"
    );
    assert!(
        moving
            .iter()
            .all(|h| h.source == ScrollSource::Finger && h.y > 0.0),
        "the glide goes the way the fingers went: {glide:?}"
    );
    assert_eq!(
        all.iter()
            .filter(|h| h.phase == GesturePhase::Ended)
            .count(),
        1,
        "one end, when the glide is over"
    );
}

#[test]
fn a_slow_lift_ends_the_eased_run_at_once_with_no_glide() {
    let mut harness = harness();
    flick(&mut harness, 0.2, 4);
    assert_eq!(
        heard(&EASED).last().map(|h| h.phase),
        Some(GesturePhase::Ended)
    );
    let before = heard(&EASED).len();
    harness.advance(Duration::from_millis(500));
    assert_eq!(heard(&EASED).len(), before, "nothing coasts");
}

#[test]
fn a_touch_during_the_glide_ends_it_before_the_next_run_begins() {
    let mut harness = harness();
    flick(&mut harness, 30.0, 6);
    harness.advance(Duration::from_millis(100));
    harness.send(Input::fingers(AT, Px(0.0), Px(5.0), GesturePhase::Began));
    let all = heard(&EASED);
    let tail: Vec<GesturePhase> = phases(&all).into_iter().rev().take(2).collect();
    assert_eq!(tail, [GesturePhase::Began, GesturePhase::Ended]);
}

#[test]
fn a_wheel_under_control_is_a_zoom_delivered_whole_never_eased() {
    let mut harness = harness();
    let zoom = |harness: &mut Harness| {
        harness.send(Input::detents_held(AT, 0.0, -1.0, Modifiers::CONTROL));
    };
    zoom(&mut harness);
    zoom(&mut harness);
    let detent = |zoom: bool| Heard {
        source: ScrollSource::Wheel,
        phase: GesturePhase::Changed,
        y: 60.0,
        zoom,
    };
    let eased_now = heard(&EASED);
    assert_eq!(
        eased_now.len(),
        2,
        "one gesture per click, at once: {eased_now:?}"
    );
    assert!(
        eased_now
            .iter()
            .all(|h| h.zoom && h.y.abs() == detent(true).y)
    );
    assert_eq!(heard(&AS_RECEIVED), eased_now, "as-received hears the same");
    harness.advance(Duration::from_millis(400));
    assert_eq!(heard(&EASED), eased_now, "no eased frames follow a zoom");
}

#[test]
fn a_wheel_without_control_is_still_eased_over_frames() {
    let mut harness = harness();
    harness.send(Input::detents(AT, 0.0, -1.0));
    let first: f32 = heard(&EASED).iter().map(|h| h.y).sum();
    assert!(
        first.abs() < 1.0,
        "next to nothing before time passes: {first}"
    );
    for _ in 0..30 {
        harness.advance(FRAME);
    }
    let eased = heard(&EASED);
    let sum: f32 = eased.iter().map(|h| h.y).sum();
    assert!(eased.len() > 3, "spread over frames: {eased:?}");
    assert!((sum.abs() - 60.0).abs() < 0.01, "a detent in all: {sum}");
    assert!(
        eased
            .iter()
            .all(|h| h.source == ScrollSource::Wheel && !h.zoom)
    );
}
