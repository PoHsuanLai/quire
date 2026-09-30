//! Where a sheet stands on a real Blitz document (design/30 section 2.5): `Attach::Window` hangs
//! from the top edge of its root, `Attach::Centre` in the middle of it, `Attach::Bottom` 8 above
//! the bottom edge and never taller than half the root, so Edit Widgets' desktop rows stay in
//! view; and a sheet dims nothing.

use dioxus::prelude::*;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::{Appearance, Attach, Ds, Material, RootExtent, Sheet};
use ds_blitz::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1280,
    height: 800,
    scale_percent: 100,
};

thread_local! {
    static ATTACH: Cell<Attach> = const { Cell::new(Attach::Window) };
    static WIDTH: Cell<SheetWidth> = const { Cell::new(SheetWidth::Regular) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            Sheet {
                label: "Sheet",
                onclose: |_| {},
                attach: ATTACH.with(Cell::get),
                width: WIDTH.with(Cell::get),
                div { style: "height:900px", "content" }
            }
        }
    }
}

fn start(attach: Attach, width: SheetWidth) -> Harness {
    ATTACH.with(|cell| cell.set(attach));
    WIDTH.with(|cell| cell.set(width));
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.attr(".ds-sheet", "data-presence").as_deref(),
        Some("present")
    );
    harness
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

#[test]
fn a_window_sheet_hangs_from_the_top_edge_centred_across() {
    let harness = start(Attach::Window, SheetWidth::Regular);
    let sheet = harness.rect(".ds-sheet").expect("the sheet");
    assert!(near(sheet.origin.y.0, 0.0), "{sheet:?}");
    assert!(near(sheet.size.width.0, 560.0), "{sheet:?}");
    assert!(
        near(sheet.origin.x.0 + sheet.size.width.0 / 2.0, 640.0),
        "{sheet:?}"
    );
    assert_eq!(harness.count(".ds-scrim"), 0, "a sheet dims nothing");
}

#[test]
fn a_centred_sheet_stands_in_the_middle_and_is_held_inside_the_root() {
    let harness = start(Attach::Centre, SheetWidth::Narrow);
    let sheet = harness.rect(".ds-sheet").expect("the sheet");
    assert!(near(sheet.size.width.0, 340.0), "{sheet:?}");
    assert!(
        near(sheet.origin.y.0 + sheet.size.height.0 / 2.0, 400.0),
        "{sheet:?}"
    );
    assert!(
        sheet.size.height.0 <= 800.0 - 72.0 + 0.5,
        "the 36 px inset holds: {sheet:?}"
    );
}

#[test]
fn a_bottom_sheet_stands_above_the_bottom_edge_and_under_half_the_root() {
    let harness = start(Attach::Bottom, SheetWidth::Wide);
    let sheet = harness.rect(".ds-sheet").expect("the sheet");
    assert!(near(sheet.size.width.0, 1040.0), "{sheet:?}");
    assert!(
        near(sheet.origin.x.0 + sheet.size.width.0 / 2.0, 640.0),
        "{sheet:?}"
    );
    let bottom = sheet.origin.y.0 + sheet.size.height.0;
    assert!(near(800.0 - bottom, 8.0), "8 above the bottom: {sheet:?}");
    assert!(
        near(sheet.size.height.0, 400.0 - 8.0),
        "held to half the root less 8: {sheet:?}"
    );
    assert!(
        sheet.origin.y.0 >= 400.0 - 0.5,
        "the top half stays in view: {sheet:?}"
    );
}
