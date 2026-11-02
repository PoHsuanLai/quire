//! A `VirtualList` the viewer is scrolling stays where the viewer put it: rows that scroll into
//! the window are there at once (no entrance), rows arriving above leave the rows being looked at
//! where they were, a change above the cursor does not pull the list back to it, and the cursor
//! moving still does.

use dioxus::prelude::*;
use ds::components::lists::virtual_list::{RowHeight, VirtualList};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
/// The list is 200 px tall: ten 20 px rows show.
const SHOWN: usize = 10;

#[allow(non_snake_case)]
fn Page() -> Element {
    // Keys count down from 999 to 100 so a row's key is stable and a prepend is a smaller key.
    let mut keys = use_signal(|| (100..1000u32).rev().collect::<Vec<u32>>());
    let mut cursor = use_signal(|| None::<u32>);
    let row = Callback::new(|key: u32| rsx! { p { class: "item r{key}", "{key}" } });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualList::<u32> {
                    label: "Rows",
                    keys: keys(),
                    row,
                    height: RowHeight::Fixed(Px(20.0)),
                    cursor: cursor(),
                    onselect: move |key| cursor.set(Some(key)),
                }
            }
            button { class: "cursor-995", onclick: move |_| cursor.set(Some(995)), "995" }
            button { class: "cursor-500", onclick: move |_| cursor.set(Some(500)), "500" }
            button {
                class: "prepend-3",
                onclick: move |_| keys.with_mut(|keys| keys.splice(0..0, [1002, 1001, 1000]).for_each(drop)),
                "prepend"
            }
            button {
                class: "drop-first-3",
                onclick: move |_| keys.with_mut(|keys| { keys.drain(0..3); }),
                "drop"
            }
        }
    }
}

fn harness() -> Harness {
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("laid out");
    harness.send(Input::click(at));
}

fn scroll_rows(harness: &mut Harness, rows: f32) {
    let at = harness.centre(".ds-scroller").expect("the scroller");
    harness.send(Input::wheel(at, Px(0.0), Px(-20.0 * rows)));
}

/// The mounted rows' keys, in order.
fn mounted(harness: &Harness) -> Vec<u32> {
    harness.with_doc(|doc| {
        doc.query_selector_all(".item")
            .map(|found| {
                found
                    .into_iter()
                    .filter_map(|id| doc.get_node(id)?.text_content().trim().parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// How far the list is scrolled.
fn offset(harness: &Harness) -> f32 {
    harness.with_doc(|doc| {
        doc.query_selector(".ds-scroller")
            .ok()
            .flatten()
            .and_then(|id| doc.get_node(id).map(|node| node.scroll_offset().y as f32))
            .unwrap_or(0.0)
    })
}

/// Where row `key` is in the list's viewport, if it is mounted.
fn y_of(harness: &Harness, key: u32) -> Option<f32> {
    let list = harness.rect(".ds-scroller")?;
    let row = harness.rect(&format!(".r{key}"))?;
    Some(row.origin.y.0 - list.origin.y.0 - offset(harness))
}

#[test]
fn rows_scrolled_into_the_window_mount_present() {
    let mut harness = harness();
    scroll_rows(&mut harness, 150.0);
    for _ in 0..2 {
        assert_eq!(
            harness.count(".ds-list-item[data-presence=entering]"),
            0,
            "a row that scrolled in does not play an entrance"
        );
        harness.advance(std::time::Duration::from_millis(300));
    }
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(999 - 150 + 10), "rows came in");
    assert_eq!(rows.len(), SHOWN + 2 * SHOWN);
}

#[test]
fn the_default_overscan_is_a_viewports_worth_of_rows() {
    let mut harness = harness();
    scroll_rows(&mut harness, 100.0);
    assert_eq!(mounted(&harness).len(), SHOWN + 2 * SHOWN);
}

#[test]
fn rows_arriving_above_a_scrolled_list_leave_the_visible_rows_where_they_are() {
    let mut harness = harness();
    scroll_rows(&mut harness, 50.0);
    // Row 949 is the 51st row: scrolled exactly to its top.
    let before = y_of(&harness, 949).expect("mounted");
    click(&mut harness, ".prepend-3");
    assert_eq!(y_of(&harness, 949), Some(before));
    harness.advance(std::time::Duration::from_millis(400));
    assert_eq!(y_of(&harness, 949), Some(before));
    // Rows leaving above shift it back the other way.
    click(&mut harness, ".drop-first-3");
    harness.advance(std::time::Duration::from_millis(400));
    assert_eq!(y_of(&harness, 949), Some(before));
}

#[test]
fn rows_arriving_at_the_top_show_when_the_list_is_at_the_top() {
    let mut harness = harness();
    click(&mut harness, ".prepend-3");
    harness.advance(std::time::Duration::from_millis(400));
    assert_eq!(y_of(&harness, 1002), Some(0.0), "the new first row shows");
}

#[test]
fn a_change_above_a_cursor_that_is_off_screen_does_not_scroll_back_to_it() {
    let mut harness = harness();
    click(&mut harness, ".cursor-995");
    scroll_rows(&mut harness, 400.0);
    let first = mounted(&harness)[SHOWN];
    let before = y_of(&harness, first).expect("mounted");
    click(&mut harness, ".prepend-3");
    harness.advance(std::time::Duration::from_millis(400));
    assert_eq!(y_of(&harness, first), Some(before));
    assert!(
        !mounted(&harness).contains(&995),
        "the cursor row is still far away"
    );
}

#[test]
fn moving_the_cursor_still_reveals_its_row() {
    let mut harness = harness();
    click(&mut harness, ".cursor-500");
    assert!(mounted(&harness).contains(&500));
    click(&mut harness, ".cursor-995");
    assert!(mounted(&harness).contains(&995));
}
