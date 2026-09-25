//! mailo gaps 2, step 3: the send pill's failed moods and busy ring on a real Blitz document.
//! A mood's one-shot plays once and settles at `ds::settle` while the pill stays up; a spinning
//! ring turns where a draining one holds still.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Button, ButtonVariant, Ds, Fraction, Material, MotionLevel, SendMood,
    SendPhase, SendPill, SendRing, StaggerIndex, settle,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use probe::pixels;
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A pill on a stage, and buttons that set its mood.
#[allow(non_snake_case)]
fn Moods() -> Element {
    let mut mood = use_signal(|| SendMood::Calm);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; gap:8px; padding:8px",
                Button { id: "calm", variant: ButtonVariant::Mini, label: "Calm", onclick: move |_| mood.set(SendMood::Calm) }
                Button { id: "nudge", variant: ButtonVariant::Mini, label: "Nudge", onclick: move |_| mood.set(SendMood::Nudge) }
            }
            div { style: "position:relative; width:460px; height:160px",
                SendPill { text: "Not sent yet · will try again", progress: Fraction(400), phase: SendPhase::Counting, mood: mood(), onundo: |_| {} }
            }
        }
    }
}

fn pulsing(harness: &Harness) -> bool {
    harness.has_class(".ds-send-pill", "a-nudge")
}

/// Nudge plays once when the mood arrives and is at rest again at `settle(Nudge)`; the pill
/// stays up throughout.
#[test]
fn a_nudge_plays_once_settles_and_the_pill_stays() {
    let mut harness = Harness::new(Moods, VIEW);
    harness.advance(ms(100));
    assert_eq!(
        harness.attr(".ds-send-pill", "data-shown").as_deref(),
        Some("shown")
    );
    assert!(!pulsing(&harness), "no one-shot on mount");

    // Marked before the click that starts the pulse's settle timer, so nothing but real
    // overhead is spent before this instant: the comparison against `settles` below stays a
    // true lower bound.
    let pulsed = Instant::now();
    harness.click(harness.centre("#nudge").expect("the nudge button"));
    harness.advance(ms(16));
    assert!(pulsing(&harness), "{}", harness.html());
    assert_eq!(
        harness.attr(".ds-send-pill", "data-pulse").as_deref(),
        Some("a")
    );
    assert_eq!(
        harness.attr(".ds-send-pill", "data-mood").as_deref(),
        Some("nudge")
    );

    let settles = settle(Anim::Nudge, MotionLevel::Standard, StaggerIndex::new(0));
    // Half the settle, not `settles - 100ms` (fixed 2026-09-25, FINDINGS "Timing tests"): the
    // old margin was 100 ms of a 520 ms window (81 % through it), so a loaded machine's
    // overshoot on `advance` could cross the boundary before this read.
    harness.advance(settles / 2);
    assert!(pulsing(&harness), "still playing at half the settle");
    let rested = settle_until(&mut harness, |h| !pulsing(h));
    assert!(
        rested.duration_since(pulsed) >= settles,
        "the pulse rested only once the full settle had run: {:?}",
        rested.duration_since(pulsed)
    );
    assert_eq!(harness.attr(".ds-send-pill", "data-pulse"), None);
    assert_eq!(
        harness.attr(".ds-send-pill", "data-shown").as_deref(),
        Some("shown")
    );
    assert_eq!(
        harness.attr(".ds-send-pill", "data-mood").as_deref(),
        Some("nudge")
    );
}

/// Back to calm and to nudge again plays it again: the pulse restarts on the other alias.
#[test]
fn a_mood_that_returns_plays_again() {
    let mut harness = Harness::new(Moods, VIEW);
    harness.advance(ms(100));
    harness.click(harness.centre("#nudge").expect("nudge"));
    harness.advance(ms(700));
    assert!(!pulsing(&harness));
    harness.click(harness.centre("#calm").expect("calm"));
    harness.advance(ms(16));
    assert!(!pulsing(&harness), "calm plays nothing");
    harness.click(harness.centre("#nudge").expect("nudge"));
    harness.advance(ms(16));
    assert!(pulsing(&harness));
    assert_eq!(
        harness.attr(".ds-send-pill", "data-pulse").as_deref(),
        Some("b")
    );
}

/// A pill with the given ring, on its own.
fn ring_app(ring: SendRing) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "position:relative; width:460px; height:160px",
                SendPill { text: "Sending…", progress: Fraction(400), phase: SendPhase::Counting, ring, onundo: |_| {} }
            }
        }
    }
}

/// The ring's pixels at two moments 250 ms apart: how many of them changed.
fn ring_change(app: fn() -> Element) -> usize {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(600));
    let first = harness.render().expect("a frame");
    let ring = probe::centred(&harness, ".ds-send-pill", ".ds-send-ring");
    let before = pixels(&first, ring, 0.0);
    harness.advance(ms(250));
    let second = harness.render().expect("a frame");
    let after = pixels(&second, ring, 0.0);
    before.iter().zip(&after).filter(|(a, b)| a != b).count()
}

#[test]
fn a_spinning_ring_turns_and_a_draining_one_holds() {
    let spinning = ring_change(|| ring_app(SendRing::Spin));
    let draining = ring_change(|| ring_app(SendRing::Drain));
    assert_eq!(draining, 0, "a held ring does not move");
    assert!(spinning > 20, "the spinning ring moved {spinning} pixels");
}
