//! Table on a real Blitz document (design/30 section 2.6): a header press asks for the sort,
//! and dragging the edge between two headers resizes the column to the left 1:1, held at its
//! least.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Point, Px, Sort, SortDirection, Table, TableColumn, TableRow};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 500,
    height: 200,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut sort = use_signal(|| None::<Sort<u8>>);
    let columns = vec![
        TableColumn::new(0u8, "Name", Px(150.0)),
        TableColumn::new(1u8, "Kind", Px(100.0)),
        TableColumn::new(2u8, "Size", Px(80.0)),
    ];
    let rows = vec![TableRow::new(
        1u8,
        "A",
        vec![rsx! { "A" }, rsx! { "x" }, rsx! { "1" }],
    )];
    let said = sort().map_or("none".to_owned(), |sort| {
        format!("{}-{:?}", sort.column, sort.direction)
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Table::<u8, u8> { label: "Files", columns, rows, sort: sort(), on_sort: move |next| sort.set(Some(next)), onselect: |_| {} }
            p { class: "sort", "{said}" }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    harness
}

#[test]
fn a_header_press_sorts_ascending_and_the_next_flips_it() {
    let mut harness = harness();
    assert_eq!(harness.text_of(".sort").as_deref(), Some("none"));
    let kind = harness
        .centre(".ds-table-head:nth-child(2) .ds-table-column")
        .expect("Kind header");
    harness.send(Input::click(kind));
    assert_eq!(
        harness.text_of(".sort").as_deref(),
        Some(&*format!("1-{:?}", SortDirection::Ascending))
    );
    harness.send(Input::click(kind));
    assert_eq!(
        harness.text_of(".sort").as_deref(),
        Some(&*format!("1-{:?}", SortDirection::Descending))
    );
}

#[test]
fn dragging_a_header_edge_resizes_the_column_and_stops_at_its_least() {
    let mut harness = harness();
    let head = |harness: &Harness| {
        harness
            .rect(".ds-table-head")
            .map_or(-1.0, |rect| rect.size.width.0)
    };
    assert_eq!(head(&harness), 150.0);
    let edge = harness.centre(".ds-table-resize").expect("edge");
    harness.send(Input::drag(
        edge,
        Point {
            x: Px(edge.x.0 + 30.0),
            y: edge.y,
        },
        6,
    ));
    assert_eq!(head(&harness), 180.0);
    let edge = harness.centre(".ds-table-resize").expect("edge");
    harness.send(Input::drag(
        edge,
        Point {
            x: Px(edge.x.0 - 300.0),
            y: edge.y,
        },
        6,
    ));
    assert_eq!(head(&harness), 48.0, "held at the least");
}
