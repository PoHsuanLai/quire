//! `assert_settles_to_zero_frames` (design/26-DETAILS.md R3) must not declare a document at rest
//! while a `ds` timer is still pending, only stepped over one quiet window. On `Clock::Virtual`
//! it drains `VirtualClock::next_due` — advances straight to each pending sleep in turn — so a
//! sleep longer than `QUIET` (500 ms) is still caught, and the caller sees its effects: the found
//! shape, a 900 ms `SettleHold`-sized sleep started right before the
//! check, reproduced directly against a minimal component instead of a whole check mark.

use dioxus::prelude::*;
use ds_harness::harness::assert_settles_to_zero_frames;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 240,
    scale_percent: 100,
};

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// A held sleep longer than `QUIET`, in the same shape as the check mark's `SettleHold`: starts
/// on mount, then flips `#hold`'s `data-fired` once it is due.
#[allow(non_snake_case)]
fn LongHold() -> Element {
    let mut fired = use_signal(|| false);
    use_hook(|| {
        spawn(async move {
            ds::sleep(Duration::from_millis(900)).await;
            fired.set(true);
        });
    });
    rsx! { div { id: "hold", "data-fired": "{fired()}" } }
}

#[test]
fn assert_settles_to_zero_frames_drains_a_pending_900ms_sleep_on_the_virtual_clock() {
    let mut harness = virtual_harness(LongHold);
    assert_eq!(
        harness.attr("#hold", "data-fired").as_deref(),
        Some("false"),
        "the hold has not fired yet"
    );

    // A poll of QUIET (500 ms) windows would see nothing woke in the first one (the sleep is not
    // due until 900 ms) and return there, well before the sleep fires — the false pass this
    // helper must no longer give on the virtual clock, where every pending sleep is visible.
    assert_settles_to_zero_frames(&mut harness);

    assert_eq!(
        harness.attr("#hold", "data-fired").as_deref(),
        Some("true"),
        "the helper must not call the document settled while the 900 ms hold is still pending"
    );
}
