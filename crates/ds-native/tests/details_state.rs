//! design/26 D0b on a real Blitz document: a pending loop spins at once, a step every
//! `--t-spin-step`, for as long as its operation runs and goes quiet the moment it ends; a
//! failure shakes once per stamp and never for the same stamp; Reduced keeps the loop turning and
//! plays no shake (R3, R4, R6, R7).

use dioxus::prelude::*;
use ds::detail::{
    Detailed, EventStamp, Moment, Operation, PendingLayers, PendingSpec, PendingStyle,
    PendingToken, Touch, use_detail, use_pending, use_shake,
};
use ds::{Appearance, Ds, Material, Motion};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 160,
    scale_percent: 100,
};

/// A network item's state, as the bar would keep it.
#[derive(Debug, Clone, PartialEq)]
enum Net {
    Off,
    Failed(EventStamp),
}

impl Detailed for Net {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (_, Net::Failed(_)) => Moment::Failure,
            (_, Net::Off) => Moment::Unavailable,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Net::Off | Net::Failed(_) => Moment::Rest,
        }
    }
}

static NET: GlobalSignal<Net> = Signal::global(|| Net::Off);
static OP: GlobalSignal<Operation> = Signal::global(|| Operation::Idle);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

const WIFI: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: PendingLayers(4),
};

#[allow(non_snake_case)]
fn Item() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            ItemBody {}
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ItemBody() -> Element {
    let detail = use_detail(NET(), Touch::Remote);
    let frame = use_pending(OP(), WIFI);
    let shake = use_shake(detail.cue()).attrs();
    rsx! {
        div { id: "frame", "{frame:?}" }
        div { id: "shake", class: shake.as_ref().map(|(class, _)| class.clone()), "data-pulse": shake.map(|(_, alias)| alias) }
    }
}

fn frame(harness: &Harness) -> String {
    harness.text_of("#frame").unwrap_or_default()
}

fn set(harness: &mut Harness, net: Net) {
    harness.within(|| *NET.write() = net);
}

fn start(harness: &mut Harness) {
    harness.within(|| *OP.write() = Operation::Running(PendingToken::start()));
}

#[test]
fn a_pending_loop_spins_at_once_steps_until_it_ends_and_goes_quiet() {
    let mut harness =
        Harness::with_config(Item, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    start(&mut harness);
    harness.advance(Duration::from_millis(0));
    assert_eq!(frame(&harness), "Step(0)", "no grace: it shows at once");
    harness.advance(Duration::from_millis(83));
    assert_eq!(frame(&harness), "Step(1)");
    // It goes round a turn a second, and keeps going: there is no cap.
    harness.advance(Duration::from_millis(30_000));
    assert!(frame(&harness).starts_with("Step"), "{}", frame(&harness));
    harness.within(|| *OP.write() = Operation::Idle);
    harness.advance(Duration::from_millis(20));
    assert_eq!(frame(&harness), "Idle", "the moment it ends, it is gone");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_keeps_the_loop_turning() {
    let mut harness =
        Harness::with_config(Item, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| *MOTION.write() = Motion::Reduced);
    start(&mut harness);
    harness.advance(Duration::from_millis(0));
    let first = frame(&harness);
    harness.advance(Duration::from_millis(166));
    assert!(frame(&harness).starts_with("Step") && frame(&harness) != first);
}

fn shaking(harness: &Harness) -> bool {
    harness.has_class("#shake", "a-shake-x")
}

#[test]
fn a_failure_shakes_once_per_stamp_and_never_escalates() {
    let mut harness = Harness::new(Item, VIEW);
    set(&mut harness, Net::Failed(EventStamp(1)));
    settle_until(&mut harness, shaking);
    assert_eq!(harness.attr("#shake", "data-pulse").as_deref(), Some("a"));
    settle_until(&mut harness, |h| !shaking(h));
    // The same failure again is the same state: no replay (R6).
    set(&mut harness, Net::Failed(EventStamp(1)));
    harness.advance(Duration::from_millis(60));
    assert!(!shaking(&harness), "the same stamp replayed");
    // A new failure replays the identical shake, on the other alias.
    set(&mut harness, Net::Failed(EventStamp(2)));
    settle_until(&mut harness, shaking);
    assert_eq!(
        harness.attr("#shake", "class").as_deref(),
        Some("a-shake-x")
    );
    assert_settles_to_zero_frames(&mut harness);
    assert!(!shaking(&harness));
}

#[test]
fn reduced_plays_no_shake() {
    let mut harness = Harness::new(Item, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, Net::Failed(EventStamp(3)));
    harness.advance(Duration::from_millis(100));
    assert!(!shaking(&harness));
    assert_settles_to_zero_frames(&mut harness);
}
