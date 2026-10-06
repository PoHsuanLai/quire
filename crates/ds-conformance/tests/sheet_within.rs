//! A sheet attached to a pane (design/30 section 2.5): `Attach::Within` hangs it from the top edge
//! of the pane it is given, as a window's sheet hangs from its titlebar, centred over that pane
//! and as wide as the pane allows (`min(width, 88%)`), never the window's; whether the pane is
//! named by its element or by its rect.

use dioxus::prelude::*;
use ds::base::geometry::units::{Point, Px, Rect, Size};
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::host::measure::{Anchor, use_rect};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::cell::{Cell, RefCell};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1280,
    height: 800,
    scale_percent: 100,
};

/// How the page names its pane.
#[derive(Clone, Copy)]
enum Naming {
    Element,
    Rect,
}

thread_local! {
    static SHIFT: Cell<u64> = const { Cell::new(0) };
    static PANE_WIDTH: Cell<f32> = const { Cell::new(400.0) };
    static NAMING: RefCell<Naming> = const { RefCell::new(Naming::Element) };
    static WIDTH: Cell<SheetWidth> = const { Cell::new(SheetWidth::Regular) };
}

const PANE_LEFT: f32 = 100.0;
const PANE_TOP: f32 = 50.0;
const PANE_HEIGHT: f32 = 300.0;

#[allow(non_snake_case)]
fn Page() -> Element {
    let pane_width = PANE_WIDTH.with(Cell::get);
    let probe = use_rect();
    // The whole root moves down once it has mounted, as a page does while its content above
    // settles: the overlay's bounds were read before the move.
    let mut shift = use_signal(|| 0u64);
    use_future(move || async move {
        let by = SHIFT.with(Cell::get);
        if by > 0 {
            ds::base::time::clock::sleep(Duration::from_millis(40)).await;
            shift.set(by);
        }
    });
    let anchor = match NAMING.with(|cell| *cell.borrow()) {
        Naming::Element => probe.anchor(),
        Naming::Rect => Some(Anchor::Rect(Rect {
            origin: Point {
                x: Px(PANE_LEFT),
                y: Px(PANE_TOP),
            },
            size: Size {
                width: Px(pane_width),
                height: Px(PANE_HEIGHT),
            },
        })),
    };
    rsx! {
        div { style: "padding-top:{shift()}px",
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div {
                class: "pane",
                style: "margin:{PANE_TOP}px 0 0 {PANE_LEFT}px;width:{pane_width}px;height:{PANE_HEIGHT}px",
                onmounted: move |event| probe.on_mounted(event),
            }
            if let Some(anchor) = anchor {
                Sheet {
                    label: "Sheet",
                    onclose: |_| {},
                    attach: Attach::Within(anchor),
                    width: WIDTH.with(Cell::get),
                    div { style: "height:900px", "content" }
                }
            }
        }
        }
    }
}

fn start(pane_width: f32, naming: Naming, width: SheetWidth) -> Harness {
    PANE_WIDTH.with(|cell| cell.set(pane_width));
    NAMING.with(|cell| *cell.borrow_mut() = naming);
    WIDTH.with(|cell| cell.set(width));
    SHIFT.with(|cell| cell.set(0));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(800));
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
fn a_sheet_hangs_from_its_panes_top_edge_centred_over_the_pane() {
    for naming in [Naming::Element, Naming::Rect] {
        let harness = start(400.0, naming, SheetWidth::Regular);
        let sheet = harness.rect(".ds-sheet").expect("the sheet");
        assert!(
            near(sheet.origin.y.0, PANE_TOP + 12.0),
            "12 below the pane's top: {sheet:?}"
        );
        assert!(
            near(
                sheet.origin.x.0 + sheet.size.width.0 / 2.0,
                PANE_LEFT + 200.0
            ),
            "centred over the pane, not the window: {sheet:?}"
        );
        assert!(
            near(sheet.size.width.0, 400.0 * 0.88),
            "88% of the pane when 560 does not fit: {sheet:?}"
        );
        assert!(
            sheet.size.height.0 <= PANE_HEIGHT - 36.0 + 0.5,
            "held inside the pane's 36 px inset: {sheet:?}"
        );
    }
}

#[test]
fn a_wide_pane_gives_the_sheet_its_own_width_and_a_narrow_sheet_stays_narrow() {
    let regular = start(800.0, Naming::Element, SheetWidth::Regular);
    let sheet = regular.rect(".ds-sheet").expect("the sheet");
    assert!(near(sheet.size.width.0, 560.0), "{sheet:?}");
    assert!(
        near(
            sheet.origin.x.0 + sheet.size.width.0 / 2.0,
            PANE_LEFT + 400.0
        ),
        "{sheet:?}"
    );
    let narrow = start(400.0, Naming::Rect, SheetWidth::Narrow);
    let sheet = narrow.rect(".ds-sheet").expect("the sheet");
    assert!(near(sheet.size.width.0, 340.0), "{sheet:?}");
}

#[test]
fn a_sheet_within_a_pane_dims_nothing_and_says_it_hangs_within() {
    let harness = start(400.0, Naming::Element, SheetWidth::Regular);
    assert_eq!(harness.count(".ds-scrim"), 0);
    assert_eq!(
        harness.attr(".ds-sheet", "data-attach").as_deref(),
        Some("within")
    );
}

#[test]
fn a_sheet_stays_on_its_pane_when_the_layout_moved_after_the_overlay_was_measured() {
    SHIFT.with(|cell| cell.set(120));
    PANE_WIDTH.with(|cell| cell.set(400.0));
    NAMING.with(|cell| *cell.borrow_mut() = Naming::Element);
    WIDTH.with(|cell| cell.set(SheetWidth::Regular));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(1200));
    let pane = harness.rect(".pane").expect("the pane");
    let sheet = harness.rect(".ds-sheet").expect("the sheet");
    assert!(
        pane.origin.y.0 > PANE_TOP + 100.0,
        "the page moved: {pane:?}"
    );
    assert!(
        near(sheet.origin.y.0, pane.origin.y.0 + 12.0),
        "the sheet hangs 12 below the pane top where it is now: {sheet:?} {pane:?}"
    );
    assert!(
        near(sheet.origin.x.0, pane.origin.x.0 + 24.0),
        "{sheet:?} {pane:?}"
    );
}
