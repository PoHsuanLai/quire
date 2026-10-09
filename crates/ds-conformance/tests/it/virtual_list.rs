//! `VirtualList` over 10,000 rows mounts only the rows in the viewport and a few around it,
//! calls `row` only for those, keeps the cursor in view, reports the end coming near, and lets a
//! removed mounted row play its exit.

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
/// The list is 200 px tall: ten 20 px rows show.
const SHOWN: usize = 10;
/// The component's default.
const OVERSCAN: usize = 4;

thread_local! {
    /// The keys `row` was called for, on this test's thread.
    static BUILT: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
}

fn built() -> usize {
    BUILT.with(|built| built.borrow().len())
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut keys = use_signal(|| (0..ROWS).collect::<Vec<u32>>());
    let mut cursor = use_signal(|| None::<u32>);
    let mut ends = use_signal(|| 0u32);
    let row = Callback::new(|key: u32| {
        BUILT.with(|built| built.borrow_mut().insert(key));
        rsx! { p { class: "item", "{key}" } }
    });
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
                    near_end: move |()| ends += 1,
                }
            }
            button { class: "to-5000", onclick: move |_| cursor.set(Some(5000)), "5000" }
            button { class: "to-last", onclick: move |_| cursor.set(Some(ROWS - 1)), "last" }
            button { class: "remove-3", onclick: move |_| { keys.with_mut(|keys| { keys.remove(3); }); }, "remove" }
            p { class: "cursor", "{cursor().map_or(-1, i64::from)}" }
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

/// The numbers of the rows mounted, in order.
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

#[test]
fn only_the_window_and_its_overscan_are_mounted_and_built() {
    let harness = harness();
    let rows = mounted(&harness);
    assert_eq!(rows, (0..(SHOWN + OVERSCAN) as u32).collect::<Vec<_>>());
    assert_eq!(harness.count(".ds-list-item"), SHOWN + OVERSCAN);
    assert_eq!(built(), SHOWN + OVERSCAN, "row was called for those only");
}

#[test]
fn a_cursor_far_away_is_scrolled_into_view_and_the_window_follows() {
    let mut harness = harness();
    click(&mut harness, ".to-5000");
    // Row 5000 is below the view, so its bottom edge is aligned with the view's: rows 4991 to
    // 5000 show, with the overscan on each side.
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(4991 - OVERSCAN as u32));
    assert_eq!(rows.last().copied(), Some(5000 + OVERSCAN as u32));
    assert_eq!(rows.len(), SHOWN + 2 * OVERSCAN);
    assert!(built() < 100, "{} rows built of {ROWS}", built());

    click(&mut harness, ".to-last");
    let rows = mounted(&harness);
    assert_eq!(rows.last().copied(), Some(ROWS - 1));
    assert_eq!(rows.len(), SHOWN + OVERSCAN, "no overscan past the end");
}

#[test]
fn the_wheel_moves_the_window() {
    let mut harness = harness();
    let at = harness.centre(".ds-scroller").expect("the scroller");
    // Ten rows of 20 px.
    harness.send(Input::wheel(at, Px(0.0), Px(-200.0)));
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(10 - OVERSCAN as u32));
    assert_eq!(rows.len(), SHOWN + 2 * OVERSCAN);
}

#[test]
fn the_arrow_keys_ask_to_move_the_cursor_one_row() {
    let mut harness = harness();
    click(&mut harness, ".ds-virtual-list");
    for _ in 0..3 {
        harness.send(Input::key(ShortcutKey::Down));
    }
    // From no cursor the first Down rests on row 0.
    assert_eq!(harness.text_of(".cursor").as_deref(), Some("2"));
    harness.send(Input::key(ShortcutKey::Up));
    assert_eq!(harness.text_of(".cursor").as_deref(), Some("1"));
}

#[test]
fn a_removed_mounted_row_plays_its_exit_and_the_rows_below_heal() {
    let mut harness = harness();
    let before = mounted(&harness);
    click(&mut harness, ".remove-3");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 1);
    assert_eq!(
        mounted(&harness).len(),
        before.len() + 1,
        "the leaving row is still mounted"
    );
    // Its exit settles, the row is dropped and the rows below heal.
    harness.advance(Duration::from_millis(250));
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert!(harness.count(".ds-list-item[data-presence=healing]") > 0);
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(".ds-list-item[data-presence=healing]"), 0);
    let after = mounted(&harness);
    assert!(!after.contains(&3), "row 3 is gone");
    assert_eq!(after.len(), before.len());
}
