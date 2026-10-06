//! On a real Blitz document: a sheet its host hides fades out (`sheet-out`, --t-quick) and
//! reports `on_hidden` once the exit has settled, never before `settle(SheetOut)`; shown again
//! while leaving, it is present at once and never reports; and a centred sheet in a viewport root
//! stands in the middle of it.

use dioxus::prelude::*;
use ds::components::overlays::sheet_attach::Attach;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// The sheet's own page, in the motion setting given.
fn fading(motion: Motion) -> Harness {
    thread_local! {
        static MOTION: std::cell::Cell<Motion> = const { std::cell::Cell::new(Motion::Standard) };
    }
    MOTION.with(|cell| cell.set(motion));
    #[allow(non_snake_case)]
    fn Fading() -> Element {
        let motion = MOTION.with(std::cell::Cell::get);
        rsx! {
            Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Sheet, extent: RootExtent::Viewport,
                Sheet { label: "Power", onclose: move |_| {}, attach: Attach::Centre, p { "Shut down?" } }
            }
        }
    }
    Harness::new(Fading, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// A power menu the page shows and hides from a context signal, logging `on_hidden`.
#[allow(non_snake_case)]
fn Page() -> Element {
    let shown = use_context_provider(|| Signal::new(Shown::Visible));
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            Sheet {
                label: "Power",
                onclose: move |_| {},
                shown: shown(),
                on_hidden: move |_| log.with_mut(|log| log.push("hidden")),
                attach: Attach::Centre,
                p { "Shut down?" }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn set(harness: &mut Harness, to: Shown) {
    harness.within(|| {
        dioxus::core::Runtime::current()
            .in_scope(ScopeId::APP, || consume_context::<Signal<Shown>>().set(to))
    });
}

fn exit() -> Duration {
    settle(Anim::SheetOut, MotionLevel::Standard)
}

fn page() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("present")
    );
    harness
}

#[test]
fn a_hidden_sheet_leaves_then_reports_at_its_settle() {
    let mut harness = page();
    let asked = harness.now();
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("leaving")
    );
    // On the virtual clock a boundary is exact: one millisecond short of the settle, nothing
    // has been reported; the settle itself is the exit's own length from the hide.
    harness.advance(exit() - Duration::from_millis(21));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some(""),
        "not before the settle"
    );
    assert_eq!(harness.count(".ds-sheet"), 1, "still drawn while it leaves");
    let reported = settle_until(&mut harness, |h| {
        h.text_of(".log").as_deref() == Some("hidden")
    });
    assert_eq!(reported.duration_since(asked), exit());
    assert_eq!(harness.count(".ds-sheet"), 0, "gone once settled");
}

#[test]
fn shown_again_while_leaving_it_is_present_and_never_reports() {
    let mut harness = page();
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(100));
    set(&mut harness, Shown::Visible);
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("present"),
        "the hide was taken back: it never left"
    );
    harness.advance(exit() + Duration::from_millis(60));
    assert_eq!(harness.text_of(".log").as_deref(), Some(""));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("present")
    );
    // Hidden again, it reports once, at its own settle.
    set(&mut harness, Shown::Hidden);
    settle_until(&mut harness, |h| {
        h.text_of(".log").as_deref() == Some("hidden")
    });
    assert_eq!(harness.text_of(".log").as_deref(), Some("hidden"));
}

#[test]
fn a_centred_sheet_stands_in_the_middle_of_its_root() {
    let harness = page();
    let sheet = harness.rect(".ds-sheet").expect("the sheet is laid out");
    let middle = sheet.origin.y.0 + sheet.size.height.0 / 2.0;
    assert!(
        (middle - VIEW.height as f32 / 2.0).abs() <= 1.0,
        "the sheet's middle is at {middle}: {sheet:?}"
    );
    let across = sheet.origin.x.0 + sheet.size.width.0 / 2.0;
    assert!((across - VIEW.width as f32 / 2.0).abs() <= 1.0, "{sheet:?}");
}

#[test]
fn a_sheet_appears_and_settles_within_its_fade_and_reduced_motion_still_does() {
    for motion in [Motion::Standard, Motion::Reduced] {
        let mut harness = fading(motion);
        harness.advance(Duration::from_millis(40));
        assert_eq!(
            harness.attr(".ds-sheet", "data-presence").as_deref(),
            Some("entering"),
            "{motion:?}: still arriving"
        );
        harness.advance(settle(Anim::SheetIn, MotionLevel::Standard));
        assert_eq!(
            harness.attr(".ds-sheet", "data-presence").as_deref(),
            Some("present"),
            "{motion:?}: settled within the fade, not the old 400 ms slide"
        );
    }
}
