//! A menu grows to its rows until it meets the window edge it opens toward, less 8 px; past that
//! it scrolls inside and says so with an arrow strip at each end that has rows past it. Resting
//! the pointer on a strip scrolls the list and the strip goes once its end is reached; a key
//! brings the highlighted row into view. A menu that fits shows no strip.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// How far a floating surface keeps from the window's edge.
const MARGIN: f32 = 8.0;

/// The menu's old fixed cap.
const OLD_CAP: f32 = 320.0;

const END: &str = ".ds-menu-scroll[*|data-end=end]";
const START: &str = ".ds-menu-scroll[*|data-end=start]";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn menu_of(count: u8) -> Element {
    let items: Vec<MenuItem<u8>> = (0..count)
        .map(|value| MenuItem::new(value, format!("Item {value}")))
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            Menu::<u8> {
                placement: MenuPlacement::Context,
                anchor: Anchor::Point(Point { x: Px(100.0), y: Px(24.0) }),
                items,
                onpick: move |_: u8| {},
                onclose: move |_| {},
            }
        }
    }
}

#[allow(non_snake_case)]
fn Long() -> Element {
    menu_of(30)
}

#[allow(non_snake_case)]
fn Short() -> Element {
    menu_of(4)
}

fn settled(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    for _ in 0..10 {
        harness.advance(ms(100));
    }
    harness
}

/// The list's own offset in the document.
fn offset(harness: &Harness) -> f64 {
    harness
        .with_doc(|doc| {
            let list = doc.query_selector(".ds-menu-list").ok().flatten()?;
            Some(doc.get_node(list)?.scroll_offset().y)
        })
        .expect("the list is in the document")
}

#[test]
fn a_long_menu_grows_to_the_window_edge_and_scrolls_by_its_strips() {
    let mut harness = settled(Long);
    let menu = harness.rect(".ds-menu").expect("the menu is drawn");
    assert!(
        menu.size.height.0 > OLD_CAP,
        "no fixed cap: the menu is {} tall",
        menu.size.height.0
    );
    assert!(
        menu.origin.y.0 + menu.size.height.0 <= VIEW.height as f32 - MARGIN + 0.5,
        "the menu stops 8 px short of the window's edge: {menu:?}"
    );
    assert_eq!(harness.count(END), 1, "rows lie below: {}", harness.html());
    assert_eq!(harness.count(START), 0, "nothing lies above");

    let strip = harness.centre(END).expect("the bottom strip is laid out");
    harness.send(Input::pointer_move(strip));
    harness.advance(ms(1000));
    assert!(
        offset(&harness) > 0.0,
        "resting on the strip scrolled the list"
    );
    assert_eq!(harness.count(START), 1, "rows now lie above");

    for _ in 0..10 {
        if harness.count(END) == 0 {
            break;
        }
        harness.advance(ms(200));
    }
    assert_eq!(harness.count(END), 0, "the bottom strip goes at the end");
    assert_eq!(harness.count(START), 1, "the top strip stays");
}

#[test]
fn a_menu_that_fits_has_no_strips() {
    let harness = settled(Short);
    assert_eq!(harness.count(".ds-menu-scroll"), 0, "{}", harness.html());
}

#[test]
fn a_key_brings_the_highlighted_row_into_view() {
    let mut harness = settled(Long);
    for _ in 0..24 {
        harness.send(Input::key(ShortcutKey::Down));
        harness.advance(ms(50));
    }
    harness.advance(ms(200));
    let list = harness.rect(".ds-menu-list").expect("the list");
    let row = harness
        .rect(".ds-menu-item[*|data-selected=true]")
        .expect("a row is highlighted");
    assert!(
        row.origin.y.0 >= list.origin.y.0 - 0.5
            && row.origin.y.0 + row.size.height.0 <= list.origin.y.0 + list.size.height.0 + 0.5,
        "the highlighted row {row:?} is inside the list {list:?}"
    );
}
