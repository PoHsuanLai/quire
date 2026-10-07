//! What `VirtualList` does with a row it stops listing is the caller's to say: a mounted row
//! plays the `exit` it was given and the rows below heal, and a `change` that is a replacement
//! drops the rows at once and plays nothing.

use dioxus::prelude::*;
use ds::components::lists::virtual_list::{Change, RowHeight, VirtualList};
use ds::motion::presence::Exit;
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
/// Past any exit and the heal.
const SETTLED: Duration = Duration::from_millis(700);

thread_local! {
    /// The `exit` and `change` the page gives its list, on this test's thread.
    static GIVEN: Cell<(Exit, Change)> = const { Cell::new((Exit::Row, Change::Edit)) };
}

/// A page whose list is given `GIVEN`.
#[allow(non_snake_case)]
fn Page() -> Element {
    let (exit, change) = GIVEN.with(Cell::get);
    let mut keys = use_signal(|| (1..=10u32).collect::<Vec<u32>>());
    let row = Callback::new(|key: u32| rsx! { p { class: "item", "{key}" } });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualList::<u32> {
                    label: "Rows",
                    keys: keys(),
                    row,
                    height: RowHeight::Fixed(Px(20.0)),
                    exit,
                    change,
                }
            }
            button { class: "remove-3", onclick: move |_| keys.with_mut(|keys| keys.retain(|key| *key != 3)), "remove" }
            button { class: "replace", onclick: move |_| keys.set((11..=20u32).collect()), "replace" }
        }
    }
}

fn harness(exit: Exit, change: Change) -> Harness {
    GIVEN.with(|given| given.set((exit, change)));
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
fn a_removed_row_plays_the_row_exit_by_default() {
    let mut harness = harness(Exit::Row, Change::default());
    click(&mut harness, ".remove-3");
    assert_eq!(
        harness
            .attr(".ds-list-item[data-presence=leaving]", "data-exit")
            .as_deref(),
        Some("row")
    );
}

#[test]
fn a_removed_row_plays_the_exit_it_was_given() {
    let mut harness = harness(Exit::Fade, Change::Edit);
    click(&mut harness, ".remove-3");
    assert_eq!(
        harness
            .attr(".ds-list-item[data-presence=leaving]", "data-exit")
            .as_deref(),
        Some("fade")
    );
    harness.advance(SETTLED);
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert_eq!(mounted(&harness), vec![1, 2, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn a_replacement_removes_rows_at_once_with_no_exit_and_no_heal() {
    let mut harness = harness(Exit::Row, Change::Replace);
    click(&mut harness, ".remove-3");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert_eq!(mounted(&harness), vec![1, 2, 4, 5, 6, 7, 8, 9, 10]);
    harness.advance(Duration::from_millis(250));
    assert_eq!(harness.count(".ds-list-item[data-presence=healing]"), 0);
    assert_eq!(harness.count(".ds-list-item"), 9);
}

#[test]
fn a_replacement_swaps_every_row_for_the_new_ones() {
    let mut harness = harness(Exit::Row, Change::Replace);
    click(&mut harness, ".replace");
    assert_eq!(mounted(&harness), (11..=20).collect::<Vec<_>>());
    assert_eq!(
        harness.count(".ds-list-item"),
        10,
        "none of the old rows plays out"
    );
}

#[test]
fn an_edit_that_replaces_every_row_plays_every_row_out() {
    let mut harness = harness(Exit::Row, Change::Edit);
    click(&mut harness, ".replace");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 10);
    harness.advance(SETTLED);
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert_eq!(mounted(&harness), (11..=20).collect::<Vec<_>>());
}
