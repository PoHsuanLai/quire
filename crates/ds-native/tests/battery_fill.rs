//! design/23-WIDGETS.md section 4.1 on a real Blitz document: a battery ring fills from empty
//! to its level on mount, its percentage counting alongside and ending on the true value; a new
//! level sweeps from the old one; a new `WakeStamp` replays the fill; a charging bolt arrives
//! only once the fill has; once settled nothing moves and nothing asks for a frame (the
//! idle-frame rule); under Reduced motion the ring is at its level from the first frame.

use dioxus::prelude::*;
use ds::{
    Appearance, BatteryFigure, BatteryLevel, Ds, DurationToken, Fraction, Material, Motion,
    MotionLevel, RingMark, RootChrome, WakeStamp,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::cell::RefCell;
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

static LEVEL: GlobalSignal<Fraction> = Signal::global(|| Fraction(930));
static WAKE: GlobalSignal<WakeStamp> = Signal::global(WakeStamp::default);
/// A ring and its figure at `LEVEL` and `WAKE`.
fn stage(motion: Motion, mark: RingMark) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Widget, chrome: Some(RootChrome::Transparent),
            BatteryLevel { level: LEVEL(), mark, wake: WAKE(), label: "This computer" }
            span { id: "figure", BatteryFigure { level: LEVEL(), wake: WAKE() } }
        }
    }
}

#[allow(non_snake_case)]
fn Stage() -> Element {
    stage(Motion::Standard, RingMark::Plain)
}

#[allow(non_snake_case)]
fn ReducedStage() -> Element {
    stage(Motion::Reduced, RingMark::Plain)
}

#[allow(non_snake_case)]
fn ChargingStage() -> Element {
    stage(Motion::Standard, RingMark::Charging)
}

/// How much of the circle the arc draws, in thousandths: 0 with no arc, 1000 for the whole
/// circle, else its end's angle clockwise from twelve (the arc starts at twelve).
fn drawn(harness: &Harness) -> u16 {
    let Some(d) = harness.attr(".ds-battery-arc path", "d") else {
        return 0;
    };
    if d.ends_with('Z') {
        return 1000;
    }
    let numbers: Vec<f32> = d
        .rsplit(' ')
        .take(2)
        .filter_map(|n| n.parse().ok())
        .collect();
    let [y, x] = numbers[..] else {
        panic!("an arc path without an end: {d}");
    };
    let degrees = (x - 50.0).atan2(50.0 - y).to_degrees().rem_euclid(360.0);
    (degrees / 360.0 * 1000.0).round() as u16
}

fn moving(harness: &Harness) -> bool {
    harness.attr(".ds-battery", "data-pulse").is_some()
}

fn figure(harness: &Harness) -> String {
    harness.text_of("#figure").unwrap_or_default()
}

/// Poll until the ring is at rest, recording every arc and figure seen on the way.
fn samples(harness: &mut Harness) -> (Vec<u16>, Vec<String>, Instant) {
    let seen = RefCell::new((Vec::new(), Vec::new()));
    let rested = settle_until(harness, |h| {
        let mut seen = seen.borrow_mut();
        seen.0.push(drawn(h));
        seen.1.push(figure(h));
        !moving(h)
    });
    let (arcs, figures) = seen.into_inner();
    (arcs, figures, rested)
}

fn fill() -> Duration {
    DurationToken::Fill.duration(MotionLevel::Standard)
}

fn assert_climbs(arcs: &[u16]) {
    assert!(
        arcs.windows(2).all(|pair| pair[0] <= pair[1]),
        "the arc went back: {arcs:?}"
    );
    let distinct = arcs.windows(2).filter(|pair| pair[0] < pair[1]).count();
    assert!(distinct >= 3, "hardly moved through the fill: {arcs:?}");
}

/// Settled for good: the same arc after a while, no pulse, no frame asked for.
fn assert_rests(harness: &mut Harness) {
    let before = harness.attr(".ds-battery-arc path", "d");
    harness.advance(Duration::from_millis(300));
    assert_eq!(harness.attr(".ds-battery-arc path", "d"), before);
    assert!(!moving(harness), "{}", harness.html());
    assert!(!harness.is_animating(), "a settled ring asks for frames");
}

#[test]
fn a_ring_fills_from_empty_and_its_figure_counts_to_the_level() {
    let mounted = Instant::now();
    let mut harness = Harness::new(Stage, VIEW);
    assert_eq!(
        harness.attr(".ds-battery", "aria-valuenow").as_deref(),
        Some("93"),
        "the true level from the first frame"
    );
    let (arcs, figures, rested) = samples(&mut harness);
    assert!(
        arcs[0] < 930,
        "the fill starts short of the level: {arcs:?}"
    );
    assert_climbs(&arcs);
    assert!(
        rested.duration_since(mounted) >= fill(),
        "at rest only after the whole fill: {:?}",
        rested.duration_since(mounted)
    );
    assert!(
        (929..=931).contains(&drawn(&harness)),
        "{}",
        drawn(&harness)
    );
    assert_eq!(figure(&harness), "93%");
    let counts: Vec<u16> = figures
        .iter()
        .filter_map(|f| f.trim_end_matches('%').parse().ok())
        .collect();
    assert!(
        counts.windows(2).all(|pair| pair[0] <= pair[1]) && counts.first() < Some(&93),
        "the figure counts up: {figures:?}"
    );
    assert!(
        counts.iter().any(|&n| n > 0 && n < 93),
        "the figure passes through the numbers between: {figures:?}"
    );
    assert_rests(&mut harness);
}

#[test]
fn a_new_level_sweeps_from_the_old_and_a_new_wake_replays() {
    let mut harness = Harness::new(Stage, VIEW);
    settle_until(&mut harness, |h| !moving(h));
    harness.within(|| *LEVEL.write() = Fraction(400));
    settle_until(&mut harness, moving);
    assert_eq!(
        harness.attr(".ds-battery", "aria-valuenow").as_deref(),
        Some("40")
    );
    let (arcs, figures, _) = samples(&mut harness);
    assert!(
        arcs.windows(2).all(|pair| pair[0] >= pair[1]),
        "the arc went back up: {arcs:?}"
    );
    assert!(
        arcs.iter().any(|&a| a > 410 && a < 920),
        "no frame between the old level and the new: {arcs:?}"
    );
    assert!((399..=401).contains(&drawn(&harness)));
    assert_eq!(figure(&harness), "40%", "{figures:?}");
    assert_rests(&mut harness);

    harness.within(|| *WAKE.write() = WAKE().next());
    settle_until(&mut harness, moving);
    let (arcs, _, _) = samples(&mut harness);
    assert!(arcs[0] < 200, "a wake starts from empty: {arcs:?}");
    assert_climbs(&arcs);
    assert!((399..=401).contains(&drawn(&harness)));
    assert_eq!(figure(&harness), "40%");
    assert_rests(&mut harness);
}

#[test]
fn a_charging_bolt_arrives_after_the_fill() {
    let mounted = Instant::now();
    let mut harness = Harness::new(ChargingStage, VIEW);
    assert_eq!(
        harness.count(".ds-battery-bolt"),
        0,
        "no bolt while filling"
    );
    let bolted = settle_until(&mut harness, |h| h.count(".ds-battery-bolt") == 1);
    assert!(
        bolted.duration_since(mounted) >= fill(),
        "the bolt came before the fill ended: {:?}",
        bolted.duration_since(mounted)
    );
    settle_until(&mut harness, |h| !moving(h));
    assert_eq!(harness.attr(".ds-battery-bolt", "style"), None, "fully in");
    assert_rests(&mut harness);
    // A change while charging keeps the bolt.
    harness.within(|| *LEVEL.write() = Fraction(950));
    settle_until(&mut harness, moving);
    assert_eq!(harness.count(".ds-battery-bolt"), 1);
    settle_until(&mut harness, |h| !moving(h));
    assert_eq!(harness.count(".ds-battery-bolt"), 1);
}

#[test]
fn under_reduced_motion_the_ring_is_at_its_level_at_once() {
    let mut harness = Harness::new(ReducedStage, VIEW);
    assert!((929..=931).contains(&drawn(&harness)), "{}", harness.html());
    assert_eq!(figure(&harness), "93%");
    assert!(!moving(&harness));
    harness.within(|| *LEVEL.write() = Fraction(400));
    harness.advance(Duration::from_millis(30));
    assert!((399..=401).contains(&drawn(&harness)), "{}", harness.html());
    assert_eq!(figure(&harness), "40%");
    assert!(!moving(&harness));
    assert!(!harness.is_animating());
}
