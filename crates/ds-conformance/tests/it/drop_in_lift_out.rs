//! A surface hung from the top edge, driven by its host's `shown` through `use_presence` with
//! `Anim::DropIn` and `Exit::LiftOut`: it arrives over `--t-overview` and is present at
//! `settle(DropIn)`, leaves upward and reports `on_hidden` exactly at `settle(LiftOut)`, never
//! before; under Reduced motion both are the shorter cross-fade.

use dioxus::prelude::*;
use ds::motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The bar the page hangs: it reports `hidden` in its log once its leave has settled.
#[component]
fn Bar() -> Element {
    let shown = use_context::<Signal<Shown>>();
    let mut log = use_signal(Vec::<&'static str>::new);
    let Presented { presence, alias } = use_presence(
        shown(),
        PresenceSpec {
            enter: Anim::DropIn,
            exit: Exit::LiftOut,
        },
        Some(EventHandler::new(move |()| {
            log.with_mut(|log| log.push("hidden"))
        })),
    );
    rsx! {
        if let Some(drawn) = presence.drawn_slug() {
            div { class: "bar", "data-presence": drawn, "data-pulse": alias.slug() }
        }
        p { class: "log", {log().join(",")} }
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    use_context_provider(|| Signal::new(Shown::Visible));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport, Bar {} }
    }
}

#[allow(non_snake_case)]
fn ReducedPage() -> Element {
    use_context_provider(|| Signal::new(Shown::Visible));
    rsx! {
        Ds { appearance: Appearance { motion: Motion::Reduced, ..Appearance::default() }, material: Material::Window, extent: RootExtent::Viewport, Bar {} }
    }
}

fn set(harness: &mut Harness, to: Shown) {
    harness.within(|| {
        dioxus::core::Runtime::current()
            .in_scope(ScopeId::APP, || consume_context::<Signal<Shown>>().set(to))
    });
}

fn phase(harness: &Harness) -> Option<String> {
    harness.attr(".bar", "data-presence")
}

fn harness(page: fn() -> Element) -> Harness {
    Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

#[test]
fn the_bar_arrives_over_the_overview_flight_and_is_present_at_its_settle() {
    let mut harness = harness(Page);
    harness.advance(ms(20));
    assert_eq!(phase(&harness).as_deref(), Some("entering"));
    let flight = settle(Anim::DropIn, MotionLevel::Standard);
    assert_eq!(flight, ms(384), "350 ms and a frame's slack");
    harness.advance(flight - ms(20));
    assert_eq!(phase(&harness).as_deref(), Some("present"));
}

#[test]
fn the_bar_leaves_upward_and_reports_exactly_at_its_settle() {
    let mut harness = harness(Page);
    harness.advance(settle(Anim::DropIn, MotionLevel::Standard) + ms(60));
    let asked = harness.now();
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(20));
    assert_eq!(phase(&harness).as_deref(), Some("leaving"));
    let exit = settle(Anim::LiftOut, MotionLevel::Standard);
    harness.advance(exit - ms(21));
    assert_eq!(harness.text_of(".log").as_deref(), Some(""), "not before");
    assert_eq!(harness.count(".bar"), 1, "still drawn while it lifts out");
    let reported = settle_until(&mut harness, |h| {
        h.text_of(".log").as_deref() == Some("hidden")
    });
    assert_eq!(reported.duration_since(asked), exit);
    assert_eq!(harness.count(".bar"), 0, "gone once settled");
}

#[test]
fn under_reduced_motion_the_bar_settles_within_the_cross_fade() {
    let mut harness = harness(ReducedPage);
    harness.advance(settle(Anim::DropIn, MotionLevel::Reduced));
    assert_eq!(phase(&harness).as_deref(), Some("present"));
    assert!(
        settle(Anim::LiftOut, MotionLevel::Reduced) < settle(Anim::LiftOut, MotionLevel::Standard)
    );
}
