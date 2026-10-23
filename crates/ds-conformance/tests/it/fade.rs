//! `Fade` on a real Blitz document, on the virtual clock: its content stands as it is while shown,
//! fades out over `--t-quick` when hidden and is unmounted once it has settled (`on_hidden` runs
//! once), takes the hide back when shown again mid-fade, and under Reduced motion leaves on the
//! short reduced settle every presence uses.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
thread_local! {
    static GONE: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn stage(motion: Motion) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Window,
            Fade { shown: SHOWN(), on_hidden: move |()| GONE.set(GONE.get() + 1),
                span { id: "badge", "done" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Standard() -> Element {
    stage(Motion::Standard)
}

#[allow(non_snake_case)]
fn Reduced() -> Element {
    stage(Motion::Reduced)
}

fn harness(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    harness
}

fn set(harness: &mut Harness, shown: Shown) {
    harness.within(|| *SHOWN.write() = shown);
}

#[test]
fn hidden_it_fades_the_content_out_and_then_unmounts_it() {
    let mut harness = harness(Standard);
    assert_eq!(harness.count("#badge"), 1);
    assert_eq!(
        harness.attr(".ds-fade", "data-presence").as_deref(),
        Some("present")
    );
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(40));
    assert_eq!(harness.count("#badge"), 1, "still there while it fades");
    assert_eq!(
        harness.attr(".ds-fade", "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(
        harness.attr(".ds-fade", "aria-hidden").as_deref(),
        Some("true")
    );
    harness.advance(ms(400));
    assert_eq!(harness.count("#badge"), 0, "unmounted once settled");
    assert_eq!(GONE.get(), 1);
}

#[test]
fn shown_again_mid_fade_it_stays() {
    let mut harness = harness(Standard);
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(40));
    set(&mut harness, Shown::Visible);
    harness.advance(ms(400));
    assert_eq!(harness.count("#badge"), 1);
    assert_eq!(GONE.get(), 0);
}

#[test]
fn under_reduced_motion_it_leaves_on_the_reduced_settle() {
    let mut harness = harness(Reduced);
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(250));
    assert_eq!(harness.count("#badge"), 0);
    assert_eq!(GONE.get(), 1);
}
