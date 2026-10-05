//! A key that leaves a `VirtualList` and is listed again is drawn once: before its exit settles it
//! drops out of the leaving rows and stays for good (nothing below it heals); after it settled it
//! is an ordinary re-insert; of two keys that leave together, the one that returns stays and the
//! other still plays out and heals the rows below.

use dioxus::prelude::*;
use ds::components::lists::virtual_list::{RowHeight, VirtualList};
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
/// Part of the row-out, so an exit is still playing.
const MID_EXIT: Duration = Duration::from_millis(50);
/// Past the row-out and the heal.
const SETTLED: Duration = Duration::from_millis(700);

#[allow(non_snake_case)]
fn Page() -> Element {
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
                }
            }
            button { class: "remove-3", onclick: move |_| keys.with_mut(|keys| keys.retain(|key| *key != 3)), "remove" }
            button { class: "remove-3-4", onclick: move |_| keys.with_mut(|keys| keys.retain(|key| *key != 3 && *key != 4)), "remove two" }
            button {
                class: "restore-3",
                onclick: move |_| keys.with_mut(|keys| if !keys.contains(&3) { keys.insert(2, 3) }),
                "restore"
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
fn a_key_back_before_its_exit_settles_is_drawn_once_and_nothing_heals() {
    let mut harness = harness();
    click(&mut harness, ".remove-3");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 1);
    harness.advance(MID_EXIT);
    click(&mut harness, ".restore-3");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert_eq!(mounted(&harness), (1..=10).collect::<Vec<_>>());
    harness.advance(SETTLED);
    assert_eq!(mounted(&harness), (1..=10).collect::<Vec<_>>());
    assert_eq!(harness.count(".ds-list-item[data-presence=healing]"), 0);
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
}

#[test]
fn a_key_back_after_its_exit_settled_is_an_ordinary_insert() {
    let mut harness = harness();
    click(&mut harness, ".remove-3");
    harness.advance(SETTLED);
    assert_eq!(mounted(&harness), vec![1, 2, 4, 5, 6, 7, 8, 9, 10]);
    click(&mut harness, ".restore-3");
    assert_eq!(mounted(&harness), (1..=10).collect::<Vec<_>>());
    harness.advance(SETTLED);
    assert_eq!(mounted(&harness), (1..=10).collect::<Vec<_>>());
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
}

#[test]
fn of_two_leaving_keys_the_one_that_returns_stays_and_the_other_heals_the_rows_below() {
    let mut harness = harness();
    click(&mut harness, ".remove-3-4");
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 2);
    harness.advance(MID_EXIT);
    click(&mut harness, ".restore-3");
    assert_eq!(
        harness.count(".ds-list-item[data-presence=leaving]"),
        1,
        "only key 4 still leaves"
    );
    assert_eq!(
        mounted(&harness),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "key 4 plays out where it stood, above 5"
    );
    harness.advance(Duration::from_millis(250));
    assert_eq!(harness.count(".ds-list-item[data-presence=leaving]"), 0);
    assert_eq!(
        harness.count(".ds-list-item[data-presence=healing]"),
        6,
        "rows 5 to 10 heal one row, not two"
    );
    assert_eq!(
        harness.count(".ds-list-item[style*='--dy:20px']"),
        6,
        "by the one dropped row's height"
    );
    harness.advance(SETTLED);
    assert_eq!(mounted(&harness), vec![1, 2, 3, 5, 6, 7, 8, 9, 10]);
}
