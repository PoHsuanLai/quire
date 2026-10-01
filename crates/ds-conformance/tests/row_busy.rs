//! design/26 and design/30 section 2.9 on a real Blitz document, on the virtual clock: a settings
//! row through an operation on its item. A busy row (`Availability::Busy`) shows the small
//! spinner where its lock was, turning at once and for as long as the work runs, and takes no
//! press; its end draws the row as it is, with no flourish; a connected device's battery and an
//! output's check show at once. Each moment ends at 0 frames (R3); Reduced keeps the spinner
//! turning (R7).

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::content::status::battery_state::BatteryState;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds_harness::harness::assert_settles_to_zero_frames;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
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
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// Where an operation on a row's item stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Rest,
    Working,
    Done,
}

impl Run {
    fn availability(self) -> Availability {
        match self {
            Run::Working => Availability::Busy,
            Run::Rest | Run::Done => Availability::Enabled,
        }
    }

    fn joined(self) -> Check {
        match self {
            Run::Done => Check::On,
            Run::Rest | Run::Working => Check::Off,
        }
    }
}

static NET: GlobalSignal<Run> = Signal::global(|| Run::Rest);
static DEVICE: GlobalSignal<Run> = Signal::global(|| Run::Rest);
static OUTPUT: GlobalSignal<Run> = Signal::global(|| Run::Rest);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Rows() -> Element {
    let (net, device, output) = (NET(), DEVICE(), OUTPUT());
    let disc = match net.joined() {
        Check::On => Selection::Selected,
        Check::Off | Check::Mixed => Selection::Unselected,
    };
    let battery = match device.joined() {
        Check::On => Accessory::Battery(BatteryState {
            level: Fraction(840),
            ..BatteryState::default()
        }),
        Check::Off | Check::Mixed => Accessory::None,
    };
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            div { style: "width:340px",
                div { id: "net",
                    Row {
                        leading: RowLeading::Disc(Icon::Wifi, disc), title: "Café", size: RowSize::Settings,
                        accessory: Accessory::Glyph(Icon::Lock),
                        state: RowState { availability: net.availability(), ..RowState::default() },
                        onclick: move |_| *NET.write() = Run::Working,
                    }
                }
                div { id: "device",
                    Row {
                        leading: RowLeading::Disc(Icon::Headphones, Selection::Unselected), title: "Headphones", size: RowSize::Settings,
                        accessory: battery,
                        state: RowState { availability: device.availability(), ..RowState::default() },
                    }
                }
                div { id: "output",
                    Row {
                        leading: RowLeading::Icon(Icon::Speaker), title: "Speakers", size: RowSize::Settings,
                        accessory: Accessory::Check(output.joined()),
                        state: RowState { availability: output.availability(), ..RowState::default() },
                    }
                }
            }
        }
    }
}

fn set(harness: &mut Harness, signal: &'static GlobalSignal<Run>, run: Run) {
    harness.within(|| *signal.write() = run);
}

fn trail(harness: &Harness, row: &str) -> Option<String> {
    harness.attr(&format!("#{row} .ds-row-trailing"), "data-mark")
}

#[test]
fn working_puts_a_spinner_where_the_lock_was_at_once_and_it_turns_until_the_work_ends() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(
        trail(&harness, "net").as_deref(),
        Some("glyph"),
        "a lock is a glyph mark"
    );
    set(&mut harness, &NET, Run::Working);
    harness.advance(ms(0));
    assert_eq!(trail(&harness, "net").as_deref(), Some("busy"));
    assert_eq!(
        harness.attr("#net .ds-progress", "data-pending").as_deref(),
        Some("step"),
        "no grace: it turns at once"
    );
    assert_eq!(
        harness.attr("#net .ds-row", "aria-busy").as_deref(),
        Some("true")
    );
    // The spinner fades in over --t-quick (design/30 section 2.9) rather than popping.
    assert_eq!(
        harness.attr("#net .ds-row-spin", "data-fade").as_deref(),
        Some("in")
    );
    assert!(
        ds::stylesheet().contains(".ds-row-spin[*|data-fade=in]{ animation:fade var(--t-quick)"),
        "the stylesheet fades it in over --t-quick"
    );
    // Ten seconds on it still turns: there is no cap.
    harness.advance(ms(10_000));
    assert_eq!(
        harness.attr("#net .ds-progress", "data-pending").as_deref(),
        Some("step")
    );
    set(&mut harness, &NET, Run::Done);
    harness.advance(ms(0));
    assert_eq!(harness.count("#net .ds-progress"), 0);
    assert_eq!(
        trail(&harness, "net").as_deref(),
        Some("glyph"),
        "the lock is back"
    );
    assert_eq!(
        harness.attr("#net .ds-row-leading", "data-disc").as_deref(),
        Some("on")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_busy_row_takes_no_press() {
    let mut harness = virtual_harness(Rows);
    let at = harness.centre("#net .ds-row-title").expect("the title");
    harness.send(Input::click(at));
    harness.advance(ms(20));
    assert_eq!(
        harness.attr("#net .ds-row", "aria-busy").as_deref(),
        Some("true"),
        "the press started the work"
    );
    harness.send(Input::click(at));
    harness.advance(ms(20));
    assert_eq!(
        harness.count("#net .ds-progress"),
        1,
        "a second press did nothing"
    );
}

fn figure(harness: &Harness) -> Option<u32> {
    harness
        .text_of("#device .ds-row-figure")
        .and_then(|text| text.trim_end_matches('%').parse().ok())
}

#[test]
fn a_connected_devices_battery_shows_its_level_at_once() {
    let mut harness = virtual_harness(Rows);
    set(&mut harness, &DEVICE, Run::Working);
    harness.advance(ms(400));
    assert_eq!(harness.count("#device .ds-progress"), 1);
    set(&mut harness, &DEVICE, Run::Done);
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(84), "the number is not counted up");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn an_output_switched_shows_its_check_and_keeps_it() {
    let mut harness = virtual_harness(Rows);
    assert_eq!(
        harness.count("#output .ds-row-trailing svg"),
        0,
        "not chosen: no check"
    );
    set(&mut harness, &OUTPUT, Run::Working);
    harness.advance(ms(700));
    set(&mut harness, &OUTPUT, Run::Done);
    harness.advance(ms(16));
    assert_eq!(
        harness.count("#output .ds-row-trailing svg.ds-ic"),
        1,
        "the plain check, at once"
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_keeps_the_spinner_turning() {
    let mut harness = virtual_harness(Rows);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.advance(ms(20));
    set(&mut harness, &NET, Run::Working);
    harness.advance(ms(0));
    let turn = harness.attr("#net .ds-progress", "style");
    harness.advance(ms(83));
    assert_ne!(harness.attr("#net .ds-progress", "style"), turn);
    set(&mut harness, &NET, Run::Rest);
    set(&mut harness, &DEVICE, Run::Done);
    harness.advance(ms(0));
    assert_eq!(figure(&harness), Some(84), "the battery at once");
    assert_settles_to_zero_frames(&mut harness);
}
