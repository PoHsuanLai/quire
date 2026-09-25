//! Sheet and modal parts, Q90 and Q91, on a real Blitz document: a sheet its host hides plays
//! `sheet-out` and reports `on_hidden` at `settle(SheetOut)`, not before; shown again while
//! leaving, it enters again and never reports; and a centred sheet in a viewport root stands in
//! the middle of it.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, Material, MotionLevel, RootExtent, Sheet, SheetPlacement, Shown,
    StaggerIndex, settle,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

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
                placement: SheetPlacement::Centre,
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
    settle(
        Anim::SheetOut,
        MotionLevel::Standard,
        StaggerIndex::default(),
    )
}

fn page() -> Harness {
    let mut harness = Harness::new(Page, VIEW);
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
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(
        harness.attr(".ds-scrim", "data-presence").as_deref(),
        Some("leaving")
    );
    harness.advance(exit() - Duration::from_millis(60));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some(""),
        "not before the settle"
    );
    assert_eq!(harness.count(".ds-sheet"), 1, "still drawn while it leaves");
    harness.advance(Duration::from_millis(80));
    assert_eq!(harness.text_of(".log").as_deref(), Some("hidden"));
    assert_eq!(harness.count(".ds-sheet"), 0, "gone once settled");
}

#[test]
fn shown_again_while_leaving_it_enters_and_never_reports() {
    let mut harness = page();
    set(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(100));
    set(&mut harness, Shown::Visible);
    harness.advance(Duration::from_millis(20));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("entering")
    );
    harness.advance(exit() + Duration::from_millis(600));
    assert_eq!(harness.text_of(".log").as_deref(), Some(""));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("present")
    );
    // Hidden again, it reports once, at its own settle.
    set(&mut harness, Shown::Hidden);
    harness.advance(exit() + Duration::from_millis(40));
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
