//! design/26 D0b on a real Blitz document: a pending loop waits out its grace, steps, holds its
//! still frame at its deadline and ends at 0 frames; a success check draws and rests; a failure
//! shakes once per stamp and never for the same stamp; attention nudges once; Reduced plays no
//! loop and no shake (R3, R4, R6, R7).

use dioxus::prelude::*;
use ds::detail::{
    Deadline, Detailed, EventStamp, FirstShow, Layers, Moment, Operation, PendingSpec,
    PendingStyle, PendingToken, SettleStyle, Settling, Touch, use_detail, use_nudge, use_pending,
    use_settle, use_shake,
};
use ds::{Appearance, Ds, Material, Motion};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 240,
    height: 160,
    scale_percent: 100,
};

/// A network item's state, as the bar would keep it.
#[derive(Debug, Clone, PartialEq)]
enum Net {
    Off,
    Joining,
    Joined,
    Failed(EventStamp),
    Asking(EventStamp),
}

impl Detailed for Net {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (_, Net::Joining) => Moment::Pending,
            (Net::Joining, Net::Joined) => Moment::Success,
            (Net::Off | Net::Joined | Net::Failed(_) | Net::Asking(_), Net::Joined) => {
                Moment::Change
            }
            (_, Net::Failed(_)) => Moment::Failure,
            (_, Net::Asking(_)) => Moment::Attention,
            (_, Net::Off) => Moment::Unavailable,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Net::Joining => Moment::Pending,
            Net::Off | Net::Joined | Net::Failed(_) | Net::Asking(_) => Moment::Rest,
        }
    }
}

static NET: GlobalSignal<Net> = Signal::global(|| Net::Off);
static OP: GlobalSignal<Operation> = Signal::global(|| Operation::Idle);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

const WIFI: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: Layers(4),
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
    let detail = use_detail(NET(), FirstShow::Still, Touch::Remote);
    let frame = use_pending(OP(), WIFI);
    let settling = use_settle(detail.cue(), SettleStyle::Check);
    let shake = use_shake(detail.cue()).attrs();
    let nudge = use_nudge(detail.cue()).attrs();
    let settle_word = match settling {
        Settling::Drawing(drawn) => format!("check {}", drawn.0),
        other => other.slug().to_owned(),
    };
    rsx! {
        div { id: "frame", "{frame:?}" }
        div { id: "settle", "{settle_word}" }
        div { id: "shake", class: shake.as_ref().map(|(class, _)| class.clone()), "data-pulse": shake.map(|(_, alias)| alias) }
        div { id: "nudge", class: nudge.as_ref().map(|(class, _)| class.clone()), "data-pulse": nudge.map(|(_, alias)| alias) }
    }
}

fn frame(harness: &Harness) -> String {
    harness.text_of("#frame").unwrap_or_default()
}

fn set(harness: &mut Harness, net: Net) {
    harness.within(|| *NET.write() = net);
}

fn start(harness: &mut Harness, deadline: Deadline) -> Instant {
    harness.within(|| *OP.write() = Operation::Running(PendingToken::start(deadline)));
    Instant::now()
}

#[test]
fn a_pending_loop_waits_steps_holds_and_goes_quiet() {
    let mut harness = Harness::new(Item, VIEW);
    let asked = start(&mut harness, Deadline::within(Duration::from_millis(1600)));
    harness.advance(Duration::from_millis(100));
    if asked.elapsed() <= Duration::from_millis(200) {
        assert_eq!(frame(&harness), "Idle", "nothing shows inside the grace");
    }
    let shown = settle_until(&mut harness, |h| frame(h).starts_with("Step"));
    assert!(shown.duration_since(asked) >= Duration::from_millis(400));
    let first = frame(&harness);
    settle_until(&mut harness, |h| {
        frame(h).starts_with("Step") && frame(h) != first
    });
    let held = settle_until(&mut harness, |h| frame(h) == "Stalled");
    assert!(held.duration_since(asked) >= Duration::from_millis(1600));
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(
        frame(&harness),
        "Stalled",
        "it holds still while the operation runs"
    );
    harness.within(|| *OP.write() = Operation::Idle);
    harness.advance(Duration::from_millis(20));
    assert_eq!(frame(&harness), "Idle", "the moment it ends, it is gone");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_fast_operation_shows_no_loop_at_all() {
    let mut harness = Harness::new(Item, VIEW);
    let asked = start(&mut harness, Deadline::cap());
    harness.advance(Duration::from_millis(100));
    harness.within(|| *OP.write() = Operation::Idle);
    if asked.elapsed() < Duration::from_millis(400) {
        harness.advance(Duration::from_millis(600));
        assert_eq!(frame(&harness), "Idle");
    }
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_holds_the_still_frame_after_the_grace() {
    let mut harness = Harness::new(Item, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    start(&mut harness, Deadline::cap());
    settle_until(&mut harness, |h| frame(h) != "Idle");
    assert_eq!(frame(&harness), "Stalled");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_success_draws_its_check_holds_and_rests() {
    let mut harness = Harness::new(Item, VIEW);
    set(&mut harness, Net::Joining);
    harness.advance(Duration::from_millis(20));
    set(&mut harness, Net::Joined);
    settle_until(&mut harness, |h| {
        h.text_of("#settle")
            .is_some_and(|word| word.starts_with("check") && word != "check 1000")
    });
    settle_until(&mut harness, |h| {
        h.text_of("#settle").as_deref() == Some("check 1000")
    });
    settle_until(&mut harness, |h| {
        h.text_of("#settle").as_deref() == Some("rest")
    });
    assert_settles_to_zero_frames(&mut harness);
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
fn attention_nudges_once() {
    let mut harness = Harness::new(Item, VIEW);
    set(&mut harness, Net::Asking(EventStamp(7)));
    settle_until(&mut harness, |h| h.has_class("#nudge", "a-nudge-up"));
    settle_until(&mut harness, |h| !h.has_class("#nudge", "a-nudge-up"));
    assert_settles_to_zero_frames(&mut harness);
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
