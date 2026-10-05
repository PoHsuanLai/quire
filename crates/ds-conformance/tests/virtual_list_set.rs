//! Every item of a `VirtualList` and of a `List` says where it stands in the set
//! (`aria-posinset`, one-based, and `aria-setsize`), so a selector or a screen reader can address
//! the nth row though a virtual list mounts only a window of them.

use dioxus::prelude::*;
use ds::components::lists::list::model::ListItem;
use ds::components::lists::virtual_list::{RowHeight, VirtualList};
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: u32 = 10_000;

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut keys = use_signal(|| (0..ROWS).collect::<Vec<u32>>());
    let mut cursor = use_signal(|| None::<u32>);
    let row = Callback::new(|key: u32| rsx! { p { class: "item", "{key}" } });
    let items: Vec<ListItem<u32>> = (0..3u32)
        .map(|key| {
            ListItem::row(
                key,
                format!("Row {key}"),
                rsx! { p { class: "plain", "{key}" } },
            )
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "virtual", style: "height:200px; width:300px; display:flex; flex-direction:column",
                VirtualList::<u32> {
                    label: "Rows",
                    keys: keys(),
                    row,
                    height: RowHeight::Fixed(Px(20.0)),
                    cursor: cursor(),
                }
            }
            List::<u32> { label: "Plain", items }
            button { class: "to-5000", onclick: move |_| cursor.set(Some(5000)), "5000" }
            button { class: "remove-3", onclick: move |_| { keys.with_mut(|keys| { keys.remove(3); }); }, "remove" }
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

fn text_at(harness: &Harness, posinset: u32) -> Option<String> {
    harness
        .text_of(&format!(".virtual [aria-posinset=\"{posinset}\"]"))
        .map(|text| text.trim().to_owned())
}

#[test]
fn the_window_at_the_top_is_numbered_from_one() {
    let harness = harness();
    assert_eq!(text_at(&harness, 1).as_deref(), Some("0"));
    assert_eq!(text_at(&harness, 4).as_deref(), Some("3"));
    assert_eq!(
        harness
            .attr(".virtual .ds-list-item", "aria-setsize")
            .as_deref(),
        Some("10000")
    );
    assert_eq!(text_at(&harness, 400), None, "only the window is mounted");
}

#[test]
fn after_a_scroll_to_row_5000_the_window_carries_its_own_numbers() {
    let mut harness = harness();
    click(&mut harness, ".to-5000");
    assert_eq!(text_at(&harness, 5001).as_deref(), Some("5000"));
    assert_eq!(text_at(&harness, 4992).as_deref(), Some("4991"));
    assert_eq!(text_at(&harness, 4).as_deref(), None);
    assert_eq!(
        harness
            .attr(".virtual [aria-posinset=\"5001\"]", "aria-setsize")
            .as_deref(),
        Some("10000")
    );
}

#[test]
fn a_removal_renumbers_the_rows_below_and_shrinks_the_set() {
    let mut harness = harness();
    assert_eq!(text_at(&harness, 5).as_deref(), Some("4"));
    click(&mut harness, ".remove-3");
    // The leaving row is no longer one of the set: the row after it already stands at 4.
    assert_eq!(text_at(&harness, 4).as_deref(), Some("4"));
    assert_eq!(text_at(&harness, 5).as_deref(), Some("5"));
    assert_eq!(
        harness.count(".virtual .ds-list-item[data-presence=leaving]"),
        1
    );
    assert_eq!(
        harness.count(".virtual .ds-list-item[data-presence=leaving][aria-posinset]"),
        0
    );
    harness.advance(Duration::from_millis(700));
    assert_eq!(text_at(&harness, 4).as_deref(), Some("4"));
    assert_eq!(
        harness
            .attr(".virtual [aria-posinset=\"4\"]", "aria-setsize")
            .as_deref(),
        Some("9999")
    );
}

#[test]
fn a_lists_items_are_numbered_the_same_way() {
    let harness = harness();
    let item = |n: u32| harness.text_of(&format!(".ds-list [aria-posinset=\"{n}\"]"));
    assert_eq!(item(2).as_deref().map(str::trim), Some("1"));
    assert_eq!(item(3).as_deref().map(str::trim), Some("2"));
    assert_eq!(
        harness
            .attr(".ds-list [aria-posinset=\"1\"]", "aria-setsize")
            .as_deref(),
        Some("3")
    );
}
