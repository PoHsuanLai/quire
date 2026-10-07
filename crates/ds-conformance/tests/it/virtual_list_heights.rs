//! `VirtualList` over 10,000 rows of two heights (a 30 px heading before every three 70 px rows)
//! mounts only the window the viewport touches, scrolls the cursor's own row into view by its own
//! offset and height, follows the wheel by the sum of the heights above, and reports the end.

use dioxus::prelude::*;
use ds::components::lists::virtual_list::{RowHeight, VirtualList};
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::RefCell;
use std::collections::HashSet;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: u32 = 10_000;
/// The list is 200 px tall.
const LIST: f32 = 200.0;
/// The component's default.
const OVERSCAN: usize = 4;

thread_local! {
    /// The keys `row` was called for, on this test's thread.
    static BUILT: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
}

fn built() -> usize {
    BUILT.with(|built| built.borrow().len())
}

/// A heading (every fourth key) is 30 px, any other row 70.
fn height_of(key: u32) -> f32 {
    match key % 4 {
        0 => 30.0,
        _ => 70.0,
    }
}

/// Where a key's row starts, summed here from the heights by plain addition.
fn top_of(key: u32) -> f32 {
    (0..key).map(height_of).sum()
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut keys = use_signal(|| (0..ROWS).collect::<Vec<u32>>());
    let mut cursor = use_signal(|| None::<u32>);
    let mut ends = use_signal(|| 0u32);
    let row = Callback::new(|key: u32| {
        BUILT.with(|built| built.borrow_mut().insert(key));
        rsx! { div { class: "item", "data-key": "{key}", style: "height:100%", "{key}" } }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualList::<u32> {
                    label: "Rows",
                    keys: keys(),
                    row,
                    height: RowHeight::PerKey(Callback::new(|key: u32| Px(height_of(key)))),
                    cursor: cursor(),
                    onselect: move |key| cursor.set(Some(key)),
                    near_end: move |()| ends += 1,
                }
            }
            button { class: "remove-1", onclick: move |_| keys.with_mut(|keys| keys.retain(|key| *key != 1)), "remove" }
            button { class: "to-3", onclick: move |_| cursor.set(Some(3)), "3" }
            button { class: "to-5000", onclick: move |_| cursor.set(Some(5000)), "5000" }
            button { class: "to-last", onclick: move |_| cursor.set(Some(ROWS - 1)), "last" }
            p { class: "ends", "{ends}" }
        }
    }
}

fn harness() -> Harness {
    BUILT.with(|built| built.borrow_mut().clear());
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("laid out");
    harness.send(Input::click(at));
}

/// The keys of the rows mounted, in order.
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

/// How far the scroller element is scrolled.
fn scrolled(harness: &Harness) -> f32 {
    harness
        .with_doc(|doc| {
            let scroller = doc.query_selector(".ds-scroller").ok().flatten()?;
            Some(doc.get_node(scroller)?.scroll_offset().y as f32)
        })
        .expect("the scroller is in the document")
}

/// Where the row of `key` starts in the view, and how tall it is: its place in the document
/// less how far the scroller is scrolled.
fn placed(harness: &Harness, key: u32) -> (f32, f32) {
    let scroller = harness.rect(".ds-scroller").expect("the scroller");
    let row = harness
        .rect(&format!(".item[data-key=\"{key}\"]"))
        .unwrap_or_else(|| panic!("row {key} is mounted"));
    (
        row.origin.y.0 - scroller.origin.y.0 - scrolled(harness),
        row.size.height.0,
    )
}

#[test]
fn rows_of_two_heights_are_laid_out_at_their_own_heights() {
    let harness = harness();
    for key in 0..4 {
        assert_eq!(
            placed(&harness, key),
            (top_of(key), height_of(key)),
            "row {key}"
        );
    }
}

#[test]
fn only_the_window_and_its_overscan_are_mounted_and_built() {
    let harness = harness();
    // Rows 0 to 3 start above 200 px; row 4 starts at 240.
    assert_eq!(
        mounted(&harness),
        (0..(4 + OVERSCAN) as u32).collect::<Vec<_>>()
    );
    assert_eq!(built(), 4 + OVERSCAN, "row was called for those only");
}

#[test]
fn a_cursor_far_away_lands_on_its_own_row_at_the_view_s_bottom_edge() {
    let mut harness = harness();
    click(&mut harness, ".to-5000");
    // Row 5000 is a 30 px heading: its bottom edge is aligned with the view's.
    assert_eq!(placed(&harness, 5000), (LIST - 30.0, 30.0));
    let rows = mounted(&harness);
    assert!(rows.contains(&5000));
    assert!(rows.len() < 20, "{} rows mounted", rows.len());
    assert!(built() < 100, "{} rows built of {ROWS}", built());
}

#[test]
fn a_cursor_row_taller_than_the_gap_is_revealed_by_its_own_height() {
    let mut harness = harness();
    // Row 3 is 70 px tall and starts at 170: it hangs 40 px below the view.
    click(&mut harness, ".to-3");
    assert_eq!(placed(&harness, 3), (LIST - 70.0, 70.0));
}

#[test]
fn the_last_row_ends_the_list_with_no_overscan_past_it() {
    let mut harness = harness();
    click(&mut harness, ".to-last");
    assert_eq!(placed(&harness, ROWS - 1), (LIST - 70.0, 70.0));
    assert_eq!(mounted(&harness).last().copied(), Some(ROWS - 1));
}

#[test]
fn the_wheel_scrolls_by_the_sum_of_the_heights_above() {
    let mut harness = harness();
    let at = harness.centre(".ds-scroller").expect("the scroller");
    harness.send(Input::wheel(at, Px(0.0), Px(-600.0)));
    // 600 px down is inside row 10, which starts at 580.
    assert_eq!(placed(&harness, 10), (top_of(10) - 600.0, 70.0));
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(10 - OVERSCAN as u32));
}

#[test]
fn the_end_coming_near_is_reported_once() {
    let mut harness = harness();
    assert_eq!(harness.text_of(".ends").as_deref(), Some("0"));
    click(&mut harness, ".to-last");
    assert_eq!(harness.text_of(".ends").as_deref(), Some("1"));
    harness.advance(Duration::from_millis(300));
    assert_eq!(harness.text_of(".ends").as_deref(), Some("1"));
}

#[test]
fn a_removed_row_leaves_and_heals_the_rows_below_by_its_own_height() {
    let mut harness = harness();
    click(&mut harness, ".remove-1");
    // Row 1 is a 70 px row: it plays out at that height.
    assert_eq!(
        harness
            .attr(".ds-list-item[data-presence=leaving]", "style")
            .as_deref(),
        Some("height:70px")
    );
    harness.advance(Duration::from_millis(250));
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    // The rows below start 70 px lower than they now rest, whatever their own heights.
    let healing = ".ds-list-item[data-presence=healing]";
    assert!(harness.count(healing) > 0);
    assert_eq!(
        harness.count(&format!("{healing}[style*='--dy:70px']")),
        harness.count(healing)
    );
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(healing), 0);
    assert_eq!(
        placed(&harness, 2),
        (30.0, 70.0),
        "row 2 now follows the heading"
    );
}
