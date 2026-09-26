//! design/24-PERSONA.md section 4 on a real Blitz document: an Idle persona blinks within 6 s
//! of a wake and, 21 s after it, has nothing left playing and asks for no frame (the idle-frame
//! rule); a wince plays once; under Reduced motion a mood change plays nothing.

use dioxus::prelude::*;
use ds::{
    Appearance, BLINK_MAX, BLINK_MIN, Ds, Material, Mood, Motion, Persona, PersonaSize,
    PersonaSpec, Theme,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

const SEED: u64 = 17;

static MOOD: GlobalSignal<Mood> = Signal::global(|| Mood::Idle);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Stage() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            Persona { spec: PersonaSpec::from_seed(SEED), size: PersonaSize::Large, mood: MOOD() }
        }
    }
}

fn blinking(harness: &Harness) -> bool {
    harness.has_class(".ds-persona-blink", "a-persona-blink")
}

fn shaking(harness: &Harness) -> bool {
    harness.has_class(".ds-persona-shake", "a-persona-wince")
}

/// Any persona pulse still on an element.
fn playing(harness: &Harness) -> bool {
    const PULSES: [(&str, &str); 5] = [
        (".ds-persona-hop", "a-persona-hop"),
        (".ds-persona-shake", "a-persona-wince"),
        (".ds-persona-breath", "a-persona-breathe"),
        (".ds-persona-blink", "a-persona-blink"),
        (".ds-persona-z", "a-persona-drift"),
    ];
    PULSES
        .iter()
        .any(|(part, pulse)| harness.has_class(part, pulse))
}

#[test]
fn idle_blinks_within_six_seconds_of_a_wake_and_paints_nothing_after_twenty_one() {
    let first = PersonaSpec::from_seed(SEED)
        .blink_gaps()
        .next()
        .unwrap_or(BLINK_MIN);
    assert!((BLINK_MIN..=BLINK_MAX).contains(&first), "{first:?}");
    let woke = Instant::now();
    let mut harness = Harness::new(Stage, VIEW);
    assert!(
        harness.has_class(".ds-persona-breath", "a-persona-breathe"),
        "the wake breathes"
    );
    harness.advance(Duration::from_millis(50));
    assert!(
        harness.is_animating(),
        "a breathing persona asks for frames"
    );
    // "Not yet" at under half the first gap: no blink before the gap has passed.
    harness.advance(BLINK_MIN / 2);
    assert!(!blinking(&harness), "a blink before {BLINK_MIN:?}");
    // Up to 1.5 s before the first blink is due, then poll for it.
    harness.advance(first.saturating_sub(Duration::from_millis(1500) + BLINK_MIN / 2));
    let blinked = settle_until(&mut harness, blinking);
    let after = blinked.duration_since(woke);
    assert!(
        after >= first,
        "blinked {after:?} after the wake, gap {first:?}"
    );
    assert!(
        after <= BLINK_MAX + Duration::from_secs(1),
        "blinked only {after:?} after the wake"
    );
    let opened = settle_until(&mut harness, |h| !blinking(h));
    assert!(opened > blinked, "the blink ended");
    // The awake window is 20 s; at 21 s nothing may be playing or asking for a frame.
    let rest = Duration::from_secs(21).saturating_sub(woke.elapsed());
    harness.advance(rest);
    assert!(!playing(&harness), "a pulse left on");
    assert!(
        !harness.is_animating(),
        "the persona still animates 21 s after its wake"
    );
    harness.advance(Duration::from_millis(500));
    assert!(
        !playing(&harness) && !harness.is_animating(),
        "it woke on its own"
    );
}

#[test]
fn a_wince_shakes_once_and_holds_its_face() {
    let mut harness = Harness::new(Stage, VIEW);
    harness.advance(Duration::from_millis(300));
    assert!(!shaking(&harness), "nothing shakes on mount");
    let asked = Instant::now();
    harness.within(|| *MOOD.write() = Mood::Wince);
    settle_until(&mut harness, shaking);
    assert_eq!(
        harness.attr(".ds-persona-shake", "data-pulse").as_deref(),
        Some("a")
    );
    let stopped = settle_until(&mut harness, |h| !shaking(h));
    // `settle(PersonaWince)` is 420 + 34 ms at Standard.
    assert!(
        stopped.duration_since(asked) >= Duration::from_millis(454),
        "stopped after {:?}",
        stopped.duration_since(asked)
    );
    harness.advance(Duration::from_millis(1000));
    assert!(!shaking(&harness), "the shake played again");
    assert_eq!(
        harness.attr(".ds-persona", "data-mood").as_deref(),
        Some("wince")
    );
}

#[test]
fn under_reduced_motion_a_mood_changes_at_once_and_nothing_plays() {
    let mut harness = Harness::new(Stage, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(100));
    harness.within(|| *MOOD.write() = Mood::Happy);
    harness.advance(Duration::from_millis(100));
    assert_eq!(
        harness.attr(".ds-persona", "data-mood").as_deref(),
        Some("happy")
    );
    assert!(
        !harness.has_class(".ds-persona-hop", "a-persona-hop"),
        "a hop under Reduced"
    );
    harness.within(|| *MOOD.write() = Mood::Idle);
    harness.advance(Duration::from_millis(BLINK_MAX.as_millis() as u64 / 2));
    assert!(!blinking(&harness), "a blink under Reduced");
}
