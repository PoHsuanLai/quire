//! design/26 D2 on a real Blitz document, on the virtual clock: a settings row through an
//! operation on its item. A network joining shows a spinner where its lock was after
//! `PendingGrace`, held still at `PendingCap` (G16); the success seals the glyph's disc, springing
//! only when the row's own press started it (G17); a failure shakes the row once per stamp (G18);
//! a device connecting breathes its glyph and its battery sweeps in with the count in step (G20,
//! G21); an output switched draws its check on (G26). Each moment ends at 0 frames (R3); Reduced
//! (R7).

use dioxus::prelude::*;
use ds::detail::{EventStamp, FirstShow};
use ds::{
    Appearance, Ds, Fraction, Icon, Material, Motion, RowDisc, RowPhase, RowTrailing, RowWork,
    SettingsRow, Switch,
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

fn joined(phase: RowPhase) -> Switch {
    match phase {
        RowPhase::Succeeded(_) => Switch::On,
        RowPhase::Rest | RowPhase::Pending(_) | RowPhase::Failed(_) => Switch::Off,
    }
}

#[allow(non_snake_case)]
fn Rows() -> Element {
    let net = NET();
    let device = DEVICE();
    let output = OUTPUT();
    let disc = match joined(net) {
        Switch::On => RowDisc::On,
        Switch::Off => RowDisc::Off,
    };
    let battery = match joined(device) {
        Switch::On => RowTrailing::Battery(Fraction(840)),
        Switch::Off => RowTrailing::None,
    };
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { style: "width:340px",
                div { id: "net",
                    SettingsRow {
                        glyph: Some(Icon::Wifi), title: "Café", detail: None,
                        trailing: RowTrailing::Glyph(Icon::Lock), phase: net, work: RowWork::Trailing, disc,
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
fn joining_puts_a_spinner_where_the_lock_was_after_its_grace_and_holds_it_at_the_cap() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(trail(&harness, "net"), None, "a lock is a plain glyph mark");
    assert_eq!(harness.count("#net .ds-settings-row-trail svg.ds-ic"), 1);
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(399));
    assert_eq!(
        harness.count("#net .ds-spinner"),
        0,
        "not before PendingGrace"
    );
    harness.advance(ms(1));
    assert_eq!(trail(&harness, "net").as_deref(), Some("pending"));
    assert_eq!(
        harness.attr("#net .ds-spinner", "data-kind").as_deref(),
        Some("spin")
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
    harness.advance(ms(10_000));
    assert_eq!(
        harness.attr("#net .ds-spinner", "data-pending").as_deref(),
        Some("still")
    );
    assert_settles_to_zero_frames(&mut harness);
}

/// The disc's seal pulse class, while it plays.
fn seal(harness: &Harness) -> String {
    harness
        .attr("#net .ds-settings-row-glyph", "class")
        .unwrap_or_default()
}

#[test]
fn a_join_from_elsewhere_seals_the_disc_without_the_spring() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(2)));
    harness.advance(ms(800));
    set(&mut harness, &NET, RowPhase::Succeeded(EventStamp(2)));
    harness.advance(ms(0));
    assert!(seal(&harness).contains("a-seal-out"), "{}", seal(&harness));
    assert_eq!(
        harness
            .attr("#net .ds-settings-row-glyph", "data-disc")
            .as_deref(),
        Some("on")
    );
    assert_eq!(harness.count("#net .ds-spinner"), 0);
    assert_eq!(trail(&harness, "net"), None, "the lock is back");
    assert_settles_to_zero_frames(&mut harness);
    assert!(
        !seal(&harness).contains("a-seal-out"),
        "the seal plays once"
    );
}

#[test]
fn the_join_the_person_clicked_seals_with_the_spring_however_long_it_took() {
    let mut harness = virtual_harness(Rows);
    let at = harness
        .centre("#net .ds-settings-row-title")
        .expect("the row");
    harness.click(at);
    harness.advance(ms(0));
    assert_eq!(
        harness.within(|| *NET.read()),
        RowPhase::Pending(EventStamp(1))
    );
    // Longer than PendingCap: the press is the operation's, not stale.
    harness.advance(ms(12_000));
    set(&mut harness, &NET, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(0));
    assert!(seal(&harness).contains("a-gulp"), "{}", seal(&harness));
    assert_settles_to_zero_frames(&mut harness);
    // The press is spent: a later join from elsewhere does not spring.
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(3)));
    harness.advance(ms(500));
    set(&mut harness, &NET, RowPhase::Succeeded(EventStamp(3)));
    harness.advance(ms(0));
    assert!(seal(&harness).contains("a-seal-out"), "{}", seal(&harness));
    assert_settles_to_zero_frames(&mut harness);
}

fn shaking(harness: &Harness) -> bool {
    harness.has_class("#net .ds-settings-row", "a-shake-x")
}

#[test]
fn a_failed_join_shakes_the_row_once_per_stamp() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(4)));
    harness.advance(ms(600));
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(5)));
    harness.advance(ms(0));
    assert!(shaking(&harness));
    assert_eq!(harness.count("#net .ds-spinner"), 0);
    assert_settles_to_zero_frames(&mut harness);
    assert!(!shaking(&harness));
    // The same failure re-polled is no moment (R6).
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(5)));
    harness.advance(ms(0));
    assert!(!shaking(&harness));
    // A new failure shakes the same way.
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(6)));
    harness.advance(ms(0));
    assert!(shaking(&harness));
    assert_settles_to_zero_frames(&mut harness);
}

fn figure(harness: &Harness) -> Option<u32> {
    harness
        .text_of("#device .ds-settings-row-figure")
        .and_then(|text| text.trim_end_matches('%').parse().ok())
}

fn battery_fill(harness: &Harness) -> f32 {
    harness
        .attr("#device [*|data-part=fill] rect", "width")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[test]
fn a_device_connecting_breathes_its_glyph_then_its_battery_sweeps_in_counting_in_step() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &DEVICE, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(400));
    let glyph = "#device .ds-settings-row-glyph";
    assert_eq!(harness.attr(glyph, "data-pending").as_deref(), Some("step"));
    let first = harness.attr(glyph, "data-beat");
    harness.advance(ms(300));
    assert_ne!(
        harness.attr(glyph, "data-beat"),
        first,
        "the breath moves a half a step"
    );
    set(&mut harness, &DEVICE, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(0));
    assert_eq!(
        harness.attr(glyph, "data-pending"),
        None,
        "the breath ends with the wait"
    );
    assert_eq!(figure(&harness), Some(0), "the count starts at zero");
    harness.advance(ms(300));
    let (count, fill) = (figure(&harness).unwrap_or(0), battery_fill(&harness));
    assert!(count > 0 && count < 84, "midway: {count}");
    assert!(fill > 0.0 && fill < 9.2, "the fill moves with it: {fill}");
    harness.advance(ms(500));
    assert_eq!(figure(&harness), Some(84));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn an_output_switched_draws_its_check_on_and_keeps_it() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(
        harness.count("#output .ds-settings-row-trail svg"),
        0,
        "not chosen: no check"
    );
    set(&mut harness, &OUTPUT, RowPhase::Pending(EventStamp(1)));
    harness.advance(ms(700));
    assert_eq!(
        harness
            .attr("#output .ds-settings-row-glyph", "data-pending")
            .as_deref(),
        Some("step")
    );
    set(&mut harness, &OUTPUT, RowPhase::Succeeded(EventStamp(1)));
    harness.advance(ms(16));
    assert_eq!(
        harness.count("#output .ds-check-mark"),
        1,
        "the check draws on"
    );
    let offset = |h: &Harness| {
        h.attr("#output .ds-check-mark path", "stroke-dashoffset")
            .and_then(|v| v.parse::<f32>().ok())
    };
    let early = offset(&harness).unwrap_or(0.0);
    harness.advance(ms(100));
    let later = offset(&harness).unwrap_or(0.0);
    assert!(later < early, "drawing on: {early} then {later}");
    // Drawn over --t-move, the check holds SettleHold (900 ms), then the row rests with its
    // plain check.
    harness.advance(ms(1_100));
    assert_eq!(
        harness.count("#output .ds-check-mark"),
        0,
        "{}",
        harness.html()
    );
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(
        harness.count("#output .ds-settings-row-trail svg.ds-ic"),
        1,
        "the check stays"
    );
}

#[test]
fn reduced_holds_still_frames_does_not_shake_and_lands_at_once() {
    let mut harness = virtual_harness(Rows);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(ms(20));
    set(&mut harness, &NET, RowPhase::Pending(EventStamp(9)));
    set(&mut harness, &DEVICE, RowPhase::Pending(EventStamp(9)));
    harness.advance(ms(450));
    assert_eq!(
        harness.attr("#net .ds-spinner", "data-pending").as_deref(),
        Some("still")
    );
    assert_eq!(
        harness
            .attr("#device .ds-settings-row-glyph", "data-pending")
            .as_deref(),
        Some("still")
    );
    set(&mut harness, &NET, RowPhase::Failed(EventStamp(10)));
    set(&mut harness, &DEVICE, RowPhase::Succeeded(EventStamp(9)));
    harness.advance(ms(0));
    assert!(!shaking(&harness), "no shake under Reduced");
    assert_eq!(figure(&harness), Some(84), "the battery at once");
    assert_settles_to_zero_frames(&mut harness);
}

/// A devices pane with a connected headphone row, `first` as the pane passes it.
fn pane(first: FirstShow) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "device", style: "width:340px",
                SettingsRow { glyph: Some(Icon::Headphones), title: "Headphones", detail: None, trailing: RowTrailing::Battery(Fraction(840)), disc: RowDisc::On, first, onclick: |_| {} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn RemountedPane() -> Element {
    pane(FirstShow::Still)
}

#[allow(non_snake_case)]
fn OpenedPane() -> Element {
    pane(FirstShow::Animate)
}

#[test]
fn a_pane_remounted_in_place_shows_the_battery_still_and_one_just_opened_sweeps_it_in() {
    let mut harness = virtual_harness(RemountedPane);
    assert_eq!(
        figure(&harness),
        Some(84),
        "re-mounted in place: no Appear (R1)"
    );
    assert_settles_to_zero_frames(&mut harness);
    let mut opened = virtual_harness(OpenedPane);
    assert_eq!(
        figure(&opened),
        Some(0),
        "just opened: the count starts at zero"
    );
    opened.advance(ms(800));
    assert_eq!(figure(&opened), Some(84));
    assert_settles_to_zero_frames(&mut opened);
}
