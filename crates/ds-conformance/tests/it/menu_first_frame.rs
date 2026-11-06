//! A pop-up menu with more rows than the room under its anchor is drawn as tall as that room, and
//! it is placed at that height from the first frame it shows: it opens where it stays, rather
//! than placed by its rows' full height (pushed up from the window's edge) for a frame and then
//! dropping under its anchor. Low in the window, where there is more room above, it opens above
//! from its first frame; a short menu held under its anchor goes above once rows arrive that the
//! room below cannot show.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::{Cell, RefCell};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// About half the window lies under it.
const MIDDLE: f32 = 200.0;

/// Under 90 px lie under it: three rows, against nearly 300 above.
const LOW: f32 = 300.0;

thread_local! {
    static ANCHOR_Y: Cell<f32> = const { Cell::new(MIDDLE) };
    static OPEN: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    static ROWS: RefCell<Option<Signal<u8>>> = const { RefCell::new(None) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let open = use_signal(|| false);
    let rows = use_signal(|| 30u8);
    OPEN.with(|slot| *slot.borrow_mut() = Some(open));
    ROWS.with(|slot| *slot.borrow_mut() = Some(rows));
    let y = ANCHOR_Y.with(Cell::get);
    let items: Vec<MenuItem<u8>> = (0..rows())
        .map(|value| MenuItem::new(value, format!("Item {value}")))
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            if open() {
              Menu::<u8> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point { x: Px(100.0), y: Px(y) }),
                items,
                onpick: move |_: u8| {},
                onclose: move |_| {},
              }
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The page with its anchor at `y` and `rows` rows, settled, the menu not yet open.
fn page(y: f32, rows: u8) -> Harness {
    ANCHOR_Y.with(|cell| cell.set(y));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    set(&mut harness, |_, mut rows_now| rows_now.set(rows));
    harness.advance(ms(300));
    harness
}

/// Change the page's signals, as a key would, with no frame run.
fn set(harness: &mut Harness, change: impl FnOnce(Signal<bool>, Signal<u8>)) {
    harness.within(|| {
        let open = OPEN.with(|slot| slot.borrow().expect("app built"));
        let rows = ROWS.with(|slot| slot.borrow().expect("app built"));
        change(open, rows);
    });
    harness.key_pending(ShortcutKey::Shift);
}

/// The menu's top in each frame it is drawn in, then once it has settled.
fn tops(harness: &mut Harness) -> Vec<f32> {
    let mut tops = Vec::new();
    for _ in 0..12 {
        let shown = harness
            .attr(".ds-menu", "style")
            .is_some_and(|style| !style.contains("visibility:hidden"));
        if let (true, Some(menu)) = (shown, harness.rect(".ds-menu")) {
            tops.push(menu.origin.y.0);
        }
        harness.step_frame();
    }
    harness.advance(ms(500));
    tops.extend(harness.rect(".ds-menu").map(|menu| menu.origin.y.0));
    tops
}

/// Every top is the last one.
fn steady(tops: &[f32]) -> f32 {
    assert!(tops.len() > 1, "the menu was drawn: {tops:?}");
    let settled = tops[tops.len() - 1];
    assert!(
        tops.iter().all(|top| (top - settled).abs() < 0.5),
        "every frame draws the menu where it settles: {tops:?}"
    );
    settled
}

#[test]
fn a_capped_menu_opens_where_it_stays() {
    let mut harness = page(MIDDLE, 30);
    set(&mut harness, |mut open, _| open.set(true));
    let top = steady(&tops(&mut harness));
    assert!(top >= MIDDLE, "the menu hangs under its anchor: {top}");
}

#[test]
fn a_menu_low_in_the_window_opens_above_from_its_first_frame() {
    let mut harness = page(LOW, 30);
    set(&mut harness, |mut open, _| open.set(true));
    steady(&tops(&mut harness));
    let menu = harness.rect(".ds-menu").expect("the menu");
    assert!(
        menu.origin.y.0 + menu.size.height.0 <= LOW + 0.5,
        "the menu stands above its anchor: {menu:?}"
    );
}

#[test]
fn a_short_menu_under_its_anchor_goes_above_when_rows_arrive() {
    let mut harness = page(LOW, 2);
    set(&mut harness, |mut open, _| open.set(true));
    harness.advance(ms(500));
    let short = harness.rect(".ds-menu").expect("the menu");
    assert!(
        short.origin.y.0 >= LOW,
        "two rows fit under the anchor: {short:?}"
    );

    set(&mut harness, |_, mut rows| rows.set(30));
    harness.advance(ms(500));
    let long = harness.rect(".ds-menu").expect("the menu");
    assert!(
        long.origin.y.0 + long.size.height.0 <= LOW + 0.5,
        "thirty rows go above, where there is room for them: {long:?}"
    );
    assert!(
        long.size.height.0 > 200.0,
        "and the menu uses that room: {long:?}"
    );
}
