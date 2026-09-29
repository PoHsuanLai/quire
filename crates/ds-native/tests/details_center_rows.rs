//! design/26 on a real Blitz document, on the virtual clock: a settings row through an
//! operation on its item. A network joining shows a spinner where its lock was, turning at once
//! and for as long as the join runs; its end, a success or a failure, draws the row as it is; a
//! connected device's battery and an output's check show at once. Each moment ends at 0 frames
//! (R3); Reduced keeps the spinner turning (R7).

use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{
    Appearance, Check, Ds, Fraction, Icon, Material, Motion, RowDisc, RowPhase, RowTrailing,
    SettingsRow,
};
use ds_native::harness::assert_settles_to_zero_frames;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 200,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

static NET: GlobalSignal<RowPhase> = Signal::global(|| RowPhase::Rest);
static DEVICE: GlobalSignal<RowPhase> = Signal::global(|| RowPhase::Rest);
static OUTPUT: GlobalSignal<RowPhase> = Signal::global(|| RowPhase::Rest);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

fn joined(phase: RowPhase) -> Check {
    match phase {
        RowPhase::Succeeded(_) => Check::On,
        RowPhase::Rest | RowPhase::Pending(_) | RowPhase::Failed(_) => Check::Off,
    }
}

#[allow(non_snake_case)]
fn Rows() -> Element {
    let net = NET();
    let device = DEVICE();
    let output = OUTPUT();
    let disc = match joined(net) {
        Check::On => RowDisc::On,
        Check::Off | Check::Mixed => RowDisc::Off,
    };
    let battery = match joined(device) {
        Check::On => RowTrailing::Battery(Fraction(840)),
        Check::Off | Check::Mixed => RowTrailing::None,
    };
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { style: "width:340px",
                div { id: "net",
                    SettingsRow {
                        glyph: Some(Icon::Wifi), title: "Café", detail: None,
                        trailing: RowTrailing::Glyph(Icon::Lock), phase: net, disc,
                        onclick: move |_| *NET.write() = RowPhase::Pending(EventStamp(1)),
                    }
                }
                div { id: "device",
                    SettingsRow { glyph: Some(Icon::Headphones), title: "Headphones", detail: None, trailing: battery, phase: device, disc: RowDisc::Off, onclick: |_| {} }
                }
                div { id: "output",
                    SettingsRow { glyph: Some(Icon::Speaker), title: "Speakers", detail: None, trailing: RowTrailing::Check(joined(output)), phase: output, onclick: |_| {} }
                }
            }
        }
    }
}

fn set(harness: &mut Harness, signal: &'static GlobalSignal<RowPhase>, phase: RowPhase) {
    harness.within(|| *signal.write() = phase);
}

fn trail(harness: &Harness, row: &str) -> Option<String> {
    harness.attr(&format!("#{row} .ds-settings-row-trail"), "data-mark")
}

#[test]
fn joining_puts_a_spinner_where_the_lock_was_at_once_and_it_turns_until_the_join_ends() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(trail(&harness, "net"), None, "a lock is a plain glyph mark");
    assert_eq!(harness.count("#net .ds-settings-row-trail svg.ds-ic"), 1);
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(0));
    assert_eq!(trail(&harness, "net").as_deref(), Some("pending"));
    assert_eq!(
        harness.attr("#net .ds-spinner", "data-pending").as_deref(),
        Some("step"),
        "no grace: it turns at once"
    );
    assert_eq!(
        harness
            .attr("#net .ds-settings-row", "aria-busy")
            .as_deref(),
        Some("true")
    );
    // The glyph does not breathe: the loop is the trailing slot's.
    assert_eq!(
        harness.attr("#net .ds-settings-row-glyph", "data-pending"),
        None
    );
    // Ten seconds on it still turns: there is no cap.
    harness.advance(ms(10_000));
    assert_eq!(
        harness.attr("#net .ds-spinner", "data-pending").as_deref(),
        Some("step")
    );
    set(&mut harness, &NET, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(0));
    assert_eq!(harness.count("#net .ds-spinner"), 0);
    assert_eq!(trail(&harness, "net"), None, "the lock is back");
    assert_eq!(
        harness
            .attr("#net .ds-settings-row-glyph", "data-disc")
            .as_deref(),
        Some("on")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_failed_join_takes_the_spinner_away_and_shakes_nothing() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(4)));
    harness.advance(ms(600));
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(5)));
    harness.advance(ms(0));
    assert_eq!(harness.count("#net .ds-spinner"), 0);
    assert!(
        !harness
            .attr("#net .ds-settings-row", "class")
            .unwrap_or_default()
            .contains("a-shake"),
        "a row does not shake"
    );
    assert_settles_to_zero_frames(&mut harness);
}

fn figure(harness: &Harness) -> Option<u32> {
    harness
        .text_of("#device .ds-settings-row-figure")
        .and_then(|text| text.trim_end_matches('%').parse().ok())
}

#[test]
fn a_connected_devices_battery_shows_its_level_at_once() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &DEVICE, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr("#device .ds-settings-row-glyph", "data-pending"),
        None,
        "the glyph does not breathe"
    );
    set(&mut harness, &DEVICE, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(84), "the number is not counted up");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn an_output_switched_shows_its_check_and_keeps_it() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(
        harness.count("#output .ds-settings-row-trail svg"),
        0,
        "not chosen: no check"
    );
    set(&mut harness, &OUTPUT, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(700));
    set(&mut harness, &OUTPUT, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(16));
    assert_eq!(
        harness.count("#output .ds-settings-row-trail svg.ds-ic"),
        1,
        "the plain check, at once"
    );
    assert_eq!(harness.count("#output .ds-check-mark"), 0, "not drawn on");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_keeps_the_spinner_turning() {
    let mut harness = virtual_harness(Rows);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(ms(20));
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(9)));
    harness.advance(ms(0));
    let turn = harness.attr("#net .ds-spinner", "style");
    harness.advance(ms(83));
    assert_ne!(harness.attr("#net .ds-spinner", "style"), turn);
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(10)));
    set(&mut harness, &DEVICE, RowPhase::Succeeded(EventStamp(9)));
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(84), "the battery at once");
    assert_settles_to_zero_frames(&mut harness);
}
