//! ProgressIndicator on a real Blitz document, on the virtual clock (design/30 section 1.3 and
//! 2.9): a determinate bar moves linearly over `--t-move` to a new value and stands at its value on
//! mount (nothing sweeps in), a running spinner steps at once and only while the operation runs,
//! and a busy button shows one in its leading slot and takes no press.

use dioxus::prelude::*;
use ds::detail::{Operation, PendingToken};
use ds::{
    Appearance, Availability, Button, ControlSize, Ds, Fraction, Material, Progress,
    ProgressIndicator, ProgressStyle,
};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 120,
    scale_percent: 100,
};

static SHARE: GlobalSignal<Fraction> = Signal::global(|| Fraction(200));
static OPERATION: GlobalSignal<Operation> = Signal::global(|| Operation::Idle);
static AVAILABILITY: GlobalSignal<Availability> = Signal::global(|| Availability::Enabled);
thread_local! {
    static PRESSES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "bar", style: "width:200px",
                ProgressIndicator { progress: Progress::Known(SHARE()) }
            }
            div { id: "spin",
                ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(OPERATION()), size: ControlSize::Small }
            }
            div { id: "button",
                Button { label: "Send", availability: AVAILABILITY(), onclick: move |_| PRESSES.set(PRESSES.get() + 1) }
            }
        }
    }
}

fn harness() -> Harness {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

fn fill_width(harness: &Harness) -> f32 {
    harness
        .rect("#bar .ds-progress-fill")
        .map_or(0.0, |rect| rect.size.width.0)
}

#[test]
fn a_bar_stands_at_its_value_and_moves_linearly_over_t_move() {
    let mut harness = harness();
    let start = fill_width(&harness);
    assert!((start - 40.0).abs() < 1.0, "20 % of 200 on mount: {start}");
    harness.within(|| *SHARE.write() = Fraction(800));
    harness.advance(ms(125));
    let halfway = fill_width(&harness);
    assert!(
        (halfway - 100.0).abs() < 8.0,
        "half of --t-move (250 ms) is half of the way from 40 to 160: {halfway}"
    );
    harness.advance(ms(200));
    let done = fill_width(&harness);
    assert!((done - 160.0).abs() < 1.0, "then it stands at 80 %: {done}");
}

#[test]
fn a_spinner_steps_at_once_while_an_operation_runs_and_not_after() {
    let mut harness = harness();
    assert_eq!(
        harness
            .attr("#spin .ds-progress", "data-pending")
            .as_deref(),
        Some("idle")
    );
    harness.within(|| *OPERATION.write() = Operation::Running(PendingToken::start()));
    harness.advance(ms(0));
    assert_eq!(
        harness
            .attr("#spin .ds-progress", "data-pending")
            .as_deref(),
        Some("step"),
        "no grace: it spins at once"
    );
    let first = harness.attr("#spin .ds-progress", "style");
    harness.advance(ms(83));
    assert_ne!(
        harness.attr("#spin .ds-progress", "style"),
        first,
        "a twelfth of a turn a step"
    );
    harness.within(|| *OPERATION.write() = Operation::Idle);
    harness.advance(ms(0));
    assert_eq!(
        harness
            .attr("#spin .ds-progress", "data-pending")
            .as_deref(),
        Some("idle")
    );
}

#[test]
fn a_busy_button_shows_a_spinner_and_takes_no_press() {
    PRESSES.set(0);
    let mut harness = harness();
    assert_eq!(harness.count("#button .ds-progress"), 0);
    harness.within(|| *AVAILABILITY.write() = Availability::Busy);
    harness.advance(ms(0));
    assert_eq!(harness.count("#button .ds-button-lead .ds-progress"), 1);
    assert_eq!(
        harness.attr("#button .ds-button", "aria-busy").as_deref(),
        Some("true")
    );
    let at = harness.centre("#button .ds-button").expect("the button");
    harness.click(at);
    harness.key(ds::ShortcutKey::Enter);
    assert_eq!(PRESSES.get(), 0, "no press while busy");
    harness.within(|| *AVAILABILITY.write() = Availability::Enabled);
    harness.advance(ms(0));
    assert_eq!(
        harness.count("#button .ds-progress"),
        0,
        "the spinner goes with the work"
    );
}
