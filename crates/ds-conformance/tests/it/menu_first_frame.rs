//! A pop-up menu with more rows than the room under its anchor is drawn as tall as that room, and
//! it is placed at that height from the first frame it shows: it opens where it stays, rather
//! than placed by its rows' full height (pushed up from the window's edge) for a frame and then
//! dropping under its anchor.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// Where the anchor's bottom edge is: about half the window lies under it.
const ANCHOR_Y: f32 = 200.0;

thread_local! {
    static OPEN: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
}

#[allow(non_snake_case)]
fn Long() -> Element {
    let open = use_signal(|| false);
    OPEN.with(|slot| *slot.borrow_mut() = Some(open));
    let items: Vec<MenuItem<u8>> = (0..30)
        .map(|value| MenuItem::new(value, format!("Item {value}")))
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            if open() {
              Menu::<u8> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point { x: Px(100.0), y: Px(ANCHOR_Y) }),
                items,
                onpick: move |_: u8| {},
                onclose: move |_| {},
              }
            }
        }
    }
}

/// The menu's top in each frame it is drawn in.
fn tops(harness: &mut Harness, frames: usize) -> Vec<f32> {
    let mut tops = Vec::new();
    for _ in 0..frames {
        let shown = harness
            .attr(".ds-menu", "style")
            .is_some_and(|style| !style.contains("visibility:hidden"));
        if let (true, Some(menu)) = (shown, harness.rect(".ds-menu")) {
            tops.push(menu.origin.y.0);
        }
        harness.step_frame();
    }
    tops
}

#[test]
fn a_capped_menu_opens_where_it_stays() {
    let mut harness = Harness::new(Long, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    // The menu opens on a page already up, as one opened by a key does, one frame at a time.
    harness.within(|| OPEN.with(|slot| slot.borrow().expect("app built").set(true)));
    harness.key_pending(ShortcutKey::Shift);
    let mut tops = tops(&mut harness, 12);
    harness.advance(Duration::from_millis(500));
    tops.extend(harness.rect(".ds-menu").map(|menu| menu.origin.y.0));
    assert!(tops.len() > 1, "the menu was drawn: {tops:?}");
    let settled = tops[tops.len() - 1];
    assert!(
        settled >= ANCHOR_Y,
        "the menu hangs under its anchor: {tops:?}"
    );
    assert!(
        tops.iter().all(|top| (top - settled).abs() < 0.5),
        "every frame draws the menu where it settles: {tops:?}"
    );
}
