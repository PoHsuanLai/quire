//! Symbol effects on a real Blitz document under the virtual clock (design/35-SYMBOL-EFFECTS.md):
//! a `Once` effect fires when its trigger changes and not when the page re-renders; a `While`
//! effect runs until it is idle and then stops asking for frames; a part effect moves the part's
//! group and comes back to rest; Reduced plays no part motion.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::harness::{assert_settles_to_zero_frames, settle_until};
use ds_harness::{ClassPresence, Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

static TRIGGER: GlobalSignal<Trigger> = Signal::global(Trigger::default);
static ACTIVITY: GlobalSignal<Activity> = Signal::global(Activity::default);
static BUMP: GlobalSignal<u32> = Signal::global(|| 0);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            // An unrelated value that re-renders the page without touching any effect.
            p { id: "bump", "{BUMP()}" }
            div { id: "lid",
                Symbol { icon: Icon::Trash, size: IconSize::Large, effect: SymbolEffect::Once(OnceEffect::Part, TRIGGER()) }
            }
            div { id: "bell",
                Symbol { icon: Icon::Bell, size: IconSize::Large, effect: SymbolEffect::While(LoopEffect::Part, ACTIVITY()) }
            }
            div { id: "hop",
                Symbol { icon: Icon::Inbox, size: IconSize::Large, effect: SymbolEffect::Once(OnceEffect::Bounce, TRIGGER()) }
            }
            div { id: "spin",
                Symbol { icon: Icon::Refresh, size: IconSize::Large, effect: SymbolEffect::While(LoopEffect::Rotate, ACTIVITY()) }
            }
        }
    }
}

fn harness() -> Harness {
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// The transform of the first group in `selector`'s symbol: the part's pose, if it has one.
fn pose(harness: &Harness, selector: &str) -> Option<String> {
    harness.attr(&format!("{selector} g"), "transform")
}

#[test]
fn a_once_effect_fires_on_a_new_trigger_and_not_on_a_re_render() {
    let mut harness = harness();
    assert_eq!(pose(&harness, "#lid"), None);
    // A re-render with the same trigger fires nothing: no part motion, no class, no frames.
    harness.within(|| *BUMP.write() += 1);
    harness.advance(Duration::from_millis(100));
    assert_eq!(pose(&harness, "#lid"), None);
    assert_eq!(
        harness.has_class("#hop .ds-symbol", "a-bounce"),
        ClassPresence::Absent
    );
    assert_settles_to_zero_frames(&mut harness);
    // A new trigger fires both: the lid tips and the bounce class arrives.
    harness.within(|| *TRIGGER.write() = Trigger(1));
    settle_until(&mut harness, |h| pose(h, "#lid").is_some());
    assert_eq!(
        harness.has_class("#hop .ds-symbol", "a-bounce"),
        ClassPresence::Present
    );
    assert_eq!(
        harness.attr("#hop .ds-symbol", "data-pulse").as_deref(),
        Some("a")
    );
    // It comes back to rest by itself and the document goes quiet.
    settle_until(&mut harness, |h| pose(h, "#lid").is_none());
    harness.within(|| *BUMP.write() += 1);
    harness.advance(Duration::from_millis(100));
    assert_eq!(
        pose(&harness, "#lid"),
        None,
        "a re-render does not play it again"
    );
    assert_eq!(
        harness.attr("#hop .ds-symbol", "data-pulse").as_deref(),
        Some("a")
    );
    assert_settles_to_zero_frames(&mut harness);
    // The next trigger fires again, on the other keyframe alias.
    harness.within(|| *TRIGGER.write() = Trigger(2));
    settle_until(&mut harness, |h| pose(h, "#lid").is_some());
    assert_eq!(
        harness.attr("#hop .ds-symbol", "data-pulse").as_deref(),
        Some("b")
    );
}

#[test]
fn a_while_effect_runs_while_active_and_stops_asking_for_frames_when_idle() {
    let mut harness = harness();
    harness.within(|| *ACTIVITY.write() = Activity::Active);
    settle_until(&mut harness, |h| pose(h, "#bell").is_some());
    assert_eq!(
        harness.has_class("#spin .ds-symbol", "a-turn"),
        ClassPresence::Present
    );
    // Still going well past one cycle: it does not run out.
    harness.advance(Duration::from_secs(5));
    assert!(harness.is_animating(), "the loop keeps asking for frames");
    harness.within(|| *ACTIVITY.write() = Activity::Idle);
    settle_until(&mut harness, |h| {
        h.has_class("#spin .ds-symbol", "a-hold") == ClassPresence::Present
    });
    settle_until(&mut harness, |h| pose(h, "#bell").is_none());
    assert_eq!(
        harness.has_class("#spin .ds-symbol", "a-turn"),
        ClassPresence::Absent
    );
    assert_eq!(
        harness.has_class("#spin .ds-symbol", "a-hold"),
        ClassPresence::Present
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_motion_plays_no_part_motion() {
    let mut harness = harness();
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(Duration::from_millis(40));
    harness.within(|| *TRIGGER.write() = Trigger(1));
    for _ in 0..10 {
        harness.advance(Duration::from_millis(30));
        assert_eq!(pose(&harness, "#lid"), None);
    }
    assert_settles_to_zero_frames(&mut harness);
}
