//! `VirtualTable` over 100,000 rows asks for the cells of the rows in the viewport and a few
//! around it only, keeps its header where it is while the rows scroll, scrolls to a row the owner
//! asks for, and moves its cursor by row and by page. Its list looks mark the table, stripe every
//! other row by its index and mark the cursor's row; the plain grid is left unmarked.

use dioxus::prelude::*;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::table::look::TableLook;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::table::rules::{ColumnRules, HeaderRule};
use ds::components::lists::virtual_table::{VirtualTable, row_pitch};
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: usize = 100_000;
/// The component's default.
const OVERSCAN: usize = 4;

thread_local! {
    /// The rows `cell` was called for, on this test's thread.
    static BUILT: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
    /// The look `Looked` is drawn in, on this test's thread.
    static LOOK: Cell<TableLook> = const { Cell::new(TableLook::Plain) };
}

fn built() -> usize {
    BUILT.with(|built| built.borrow().len())
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let scroller = use_scroller();
    let mut cursor = use_signal(|| None::<usize>);
    let columns = vec![
        TableColumn::new(0u8, "Row", Px(80.0)),
        TableColumn::new(1u8, "Square", Px(120.0)),
    ];
    let cell = Callback::new(|(row, column): (usize, usize)| {
        BUILT.with(|built| built.borrow_mut().insert(row));
        match column {
            0 => rsx! { span { class: "first", "{row}" } },
            _ => rsx! { span { class: "second", "{row * row}" } },
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Squares",
                    columns,
                    rows: ROWS,
                    cell,
                    scroller,
                    on_sort: |_| {},
                    cursor: cursor(),
                    onselect: move |row| cursor.set(Some(row)),
                    rules: ColumnRules::Hairline,
                }
            }
            button { class: "to-5000", onclick: move |_| scroller.reveal_row(5000, row_pitch()), "5000" }
            p { class: "cursor", "{cursor().map_or(-1, |row| row as i64)}" }
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
fn mounted(harness: &Harness) -> Vec<usize> {
    harness.with_doc(|doc| {
        doc.query_selector_all(".first")
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
fn only_the_rows_near_the_viewport_are_mounted_and_asked_for() {
    let harness = harness();
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(0));
    assert!(
        (6..=6 + OVERSCAN + 2).contains(&rows.len()),
        "{} rows mounted",
        rows.len()
    );
    assert_eq!(harness.count(".ds-vtable-item"), rows.len());
    assert!(built() <= rows.len() + OVERSCAN, "{} cells built", built());
    assert_eq!(harness.count(".ds-table-column"), 2, "the header is there");
}

#[test]
fn a_row_the_owner_scrolls_to_is_mounted_and_the_header_stays() {
    let mut harness = harness();
    let header = harness.centre(".ds-table-header").expect("header");
    click(&mut harness, ".to-5000");
    let rows = mounted(&harness);
    assert!(rows.contains(&5000), "row 5000 is mounted: {rows:?}");
    assert!(
        rows.first().copied() > Some(4900),
        "the window followed: {rows:?}"
    );
    assert!(built() < 60, "{} cells built of {ROWS}", built());
    assert_eq!(
        harness.centre(".ds-table-header"),
        Some(header),
        "the header did not scroll"
    );
}

#[test]
fn the_wheel_moves_the_window_by_rows_of_the_pitch() {
    let mut harness = harness();
    let at = harness.centre(".ds-scroller").expect("the scroller");
    // Ten rows.
    harness.send(Input::wheel(at, Px(0.0), Px(-10.0 * row_pitch().0)));
    let rows = mounted(&harness);
    assert_eq!(rows.first().copied(), Some(10 - OVERSCAN));
}

#[test]
fn the_keys_ask_to_move_the_cursor_by_a_row_and_by_a_page_and_the_cursor_row_is_selected() {
    let mut harness = harness();
    click(&mut harness, ".ds-table-column");
    for _ in 0..3 {
        harness.send(Input::key(ShortcutKey::Down));
    }
    // From no cursor the first Down rests on row 0.
    assert_eq!(harness.text_of(".cursor").as_deref(), Some("2"));
    assert_eq!(harness.count(".ds-row[aria-selected=true]"), 1);
    harness.send(Input::key(ShortcutKey::PageDown));
    let paged: i64 = harness
        .text_of(".cursor")
        .and_then(|text| text.parse().ok())
        .expect("a row");
    assert!(paged > 3, "a page is more than a row: {paged}");
    harness.send(Input::key(ShortcutKey::End));
    assert_eq!(
        harness.text_of(".cursor").as_deref(),
        Some((ROWS - 1).to_string().as_str())
    );
    assert!(
        mounted(&harness).contains(&(ROWS - 1)),
        "the cursor is in view"
    );
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    let scroller = use_scroller();
    let columns = vec![TableColumn::new(0u8, "Row", Px(80.0))];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Bare",
                    columns,
                    rows: ROWS,
                    cell: Callback::new(|(row, _): (usize, usize)| rsx! { "{row}" }),
                    scroller,
                    on_sort: |_| {},
                    header_rule: HeaderRule::Never,
                }
            }
        }
    }
}

#[test]
fn hairline_rules_are_marked_and_none_leaves_the_table_unmarked() {
    // Read off the table itself: the page's stylesheet names both attributes too.
    let ruled = harness();
    assert_eq!(
        ruled.attr(".ds-vtable", "data-rules").as_deref(),
        Some("hairline")
    );
    let bare = Harness::new(Bare, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(
        bare.attr(".ds-vtable", "data-rules"),
        None,
        "no rules attribute"
    );
    assert_eq!(
        bare.attr(".ds-vtable", "data-scrolled"),
        None,
        "HeaderRule::Never marks nothing"
    );
}

#[test]
fn the_header_rule_shows_once_the_rows_are_scrolled() {
    let mut harness = harness();
    assert!(harness.html().contains("data-scrolled=\"no\""));
    let at = harness.centre(".ds-scroller").expect("the scroller");
    harness.send(Input::wheel(at, Px(0.0), Px(-3.0 * row_pitch().0)));
    assert!(harness.html().contains("data-scrolled=\"yes\""));
    harness.send(Input::wheel(at, Px(0.0), Px(3.0 * row_pitch().0)));
    assert!(harness.html().contains("data-scrolled=\"no\""));
}

#[allow(non_snake_case)]
fn Looked() -> Element {
    let scroller = use_scroller();
    let columns = vec![TableColumn::new(0u8, "Row", Px(80.0))];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Looked",
                    columns,
                    rows: ROWS,
                    cell: Callback::new(|(row, _): (usize, usize)| rsx! { "{row}" }),
                    scroller,
                    on_sort: |_| {},
                    cursor: Some(1),
                    look: LOOK.get(),
                }
            }
        }
    }
}

#[test]
fn a_list_look_is_marked_and_stripes_every_other_row_by_index_and_marks_the_cursor_row() {
    // (look, data-look, stripes)
    let cases = [
        (TableLook::Plain, None, false),
        (TableLook::Striped, Some("striped"), true),
        (TableLook::Unstriped, Some("unstriped"), false),
    ];
    for (look, marked, striped) in cases {
        LOOK.set(look);
        let mut harness = Harness::new(Looked, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        assert_eq!(
            harness.attr(".ds-vtable", "data-look").as_deref(),
            marked,
            "{look:?}"
        );
        assert_eq!(
            harness.count(".ds-vtable-item[data-stripe=odd]") > 0,
            striped,
            "{look:?} stripes"
        );
        assert_eq!(
            harness.count(".ds-vtable-item[data-selected=yes]"),
            1,
            "{look:?}"
        );
        assert_eq!(
            harness
                .attr(".ds-vtable-item[data-row=\"1\"]", "data-selected")
                .as_deref(),
            Some("yes"),
            "{look:?} the cursor row"
        );
        if striped {
            // The stripe follows the row, not its place among those mounted.
            assert!(
                harness
                    .attr(".ds-vtable-item[data-row=\"1\"]", "data-stripe")
                    .is_some()
            );
            assert!(
                harness
                    .attr(".ds-vtable-item[data-row=\"2\"]", "data-stripe")
                    .is_none()
            );
            let at = harness.centre(".ds-scroller").expect("the scroller");
            harness.send(Input::wheel(at, Px(0.0), Px(-5.0 * row_pitch().0)));
            let rows = mounted(&harness);
            for row in rows {
                let selector = format!(".ds-vtable-item[data-row=\"{row}\"]");
                assert_eq!(
                    harness.attr(&selector, "data-stripe").is_some(),
                    row % 2 == 1,
                    "row {row} after scrolling"
                );
            }
        }
    }
}
