//! A desktop widget's exit on a real Blitz document and the virtual clock: the host
//! says the card is leaving and keeps it drawn; the card plays `widget-out` (shrinks and fades,
//! `--t-move --e-exit`), takes no pointer, and calls `on_gone` exactly at `settle(WidgetOut)`,
//! not a millisecond before; the host drops it and nothing asks for a frame. Under Reduced the
//! exit is a fade over Reduced's short settle. Taken back before it ends, it never calls
//! `on_gone`.

use dioxus::prelude::*;
use ds::{Anim, Appearance, Ds, Material, Motion, MotionLevel, RootChrome, settle};
use ds_harness::harness::assert_settles_to_zero_frames;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use ds_shell::{
    BatteryWidget, CardPresence, Timeline, Widget, WidgetCard, WidgetMetrics, WidgetSize,
};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 240,
    scale_percent: 100,
};

static PRESENCE: GlobalSignal<CardPresence> = Signal::global(CardPresence::default);
static GONE: GlobalSignal<u32> = Signal::global(|| 0);
static DRAWN: GlobalSignal<bool> = Signal::global(|| true);

fn desktop(motion: Motion) -> Element {
    rsx! {
        Ds { sheet: Some(ds_shell::stylesheet()), appearance: Appearance { motion, ..Appearance::default() }, material: Material::Widget, chrome: Some(RootChrome::Transparent),
            div { style: "padding:20px;{WidgetMetrics::default().style_attr()}",
                if DRAWN() {
                    WidgetCard {
                        widget: BatteryWidget,
                        timeline: Timeline::now(BatteryWidget::preview(WidgetSize::Small)),
                        size: WidgetSize::Small,
                        presence: PRESENCE(),
                        on_gone: move |()| {
                            *GONE.write() += 1;
                            *DRAWN.write() = false;
                        },
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Standard() -> Element {
    desktop(Motion::Standard)
}

#[allow(non_snake_case)]
fn Reduced() -> Element {
    desktop(Motion::Reduced)
}

fn harness(app: fn() -> Element) -> Harness {
    let mut harness =
        Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(1200));
    harness
}

fn leave(harness: &mut Harness, presence: CardPresence) {
    harness.within(|| *PRESENCE.write() = presence);
    harness.advance(Duration::ZERO);
}

fn gone(harness: &mut Harness) -> u32 {
    harness.within(|| *GONE.peek())
}

#[test]
fn a_removed_card_plays_its_exit_and_is_gone_at_settle() {
    let mut harness = harness(Standard);
    assert_eq!(harness.count(".ds-widget"), 1);
    assert_eq!(harness.attr(".ds-widget", "data-presence"), None);
    assert_settles_to_zero_frames(&mut harness);
    leave(&mut harness, CardPresence::Leaving);
    assert!(harness.has_class(".ds-widget", "a-widget-out"));
    assert_eq!(
        harness.attr(".ds-widget", "data-presence").as_deref(),
        Some("leaving")
    );
    assert!(harness.is_animating(), "the exit is playing");
    let out = settle(Anim::WidgetOut, MotionLevel::Standard);
    harness.advance(out - Duration::from_millis(1));
    assert_eq!(gone(&mut harness), 0, "not before settle(WidgetOut)");
    assert_eq!(harness.count(".ds-widget"), 1, "the host keeps it drawn");
    harness.advance(Duration::from_millis(1));
    assert_eq!(gone(&mut harness), 1, "at settle(WidgetOut)");
    assert_eq!(harness.count(".ds-widget"), 0, "the host dropped it");
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(gone(&mut harness), 1, "once");
}

#[test]
fn under_reduced_the_exit_is_a_short_fade() {
    let mut harness = harness(Reduced);
    leave(&mut harness, CardPresence::Leaving);
    assert!(harness.has_class(".ds-widget", "a-widget-out"));
    let sheet = ds_shell::stylesheet();
    assert!(
        sheet.contains(
            ".ds[*|data-motion=reduced] .a-widget-out[*|data-pulse=a]{animation-name:menu-out;}"
        ),
        "Reduced plays the fade, not the shrink"
    );
    let out = settle(Anim::WidgetOut, MotionLevel::Reduced);
    let standard = settle(Anim::WidgetOut, MotionLevel::Standard);
    assert!(out < standard, "{out:?} {standard:?}");
    harness.advance(out);
    assert_eq!(gone(&mut harness), 1);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_leave_taken_back_never_reports_gone() {
    let mut harness = harness(Standard);
    leave(&mut harness, CardPresence::Leaving);
    let out = settle(Anim::WidgetOut, MotionLevel::Standard);
    harness.advance(out / 2);
    leave(&mut harness, CardPresence::Placed);
    assert!(!harness.has_class(".ds-widget", "a-widget-out"));
    assert!(
        harness.has_class(".ds-widget", "a-hold"),
        "the exit's value is dropped"
    );
    assert_eq!(harness.attr(".ds-widget", "data-presence"), None);
    harness.advance(out);
    assert_eq!(gone(&mut harness), 0);
    assert_eq!(harness.count(".ds-widget"), 1);
    assert_settles_to_zero_frames(&mut harness);
}
