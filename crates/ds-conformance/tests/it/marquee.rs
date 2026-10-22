//! A drag that begins on empty space draws a rubber band and selects what it touches, in a
//! `VirtualTable` (the rows it crosses) and in a `CollectionView` (the tiles inside it). A plain
//! drag replaces the selection and a click on empty space clears it, Shift adds to the selection
//! the drag began with, and a press on a row or a tile is not a band.

use dioxus::prelude::*;
use ds::base::selection::RowSelection;
use ds::components::content::icon_source::IconSource;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::collection_view::{CollectionItem, CollectionView, Flow, Thumbnail};
use ds::components::lists::table::look::TableLook;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::virtual_table::VirtualTable;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, PointerAction, PointerInput, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: usize = 3;
const TILES: u32 = 8;
const COLUMNS: u8 = 4;

/// The selected rows or tiles, as the page prints them.
fn printed(chosen: &RowSelection) -> String {
    chosen
        .rows()
        .map(|row| row.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn table(start: RowSelection) -> Element {
    let scroller = use_scroller();
    let mut chosen = use_signal(|| start);
    let columns = vec![TableColumn::new(0u8, "Row", Px(120.0))];
    let cell = Callback::new(|(row, _): (usize, usize)| rsx! { "{row}" });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:240px; width:300px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Rows",
                    columns,
                    rows: ROWS,
                    cell,
                    scroller,
                    on_sort: |_| {},
                    look: TableLook::Striped,
                    selection: Some(chosen()),
                    onmarquee: move |next: RowSelection| chosen.set(next),
                }
            }
            p { class: "chosen", "{printed(&chosen())}" }
        }
    }
}

#[allow(non_snake_case)]
fn EmptyTable() -> Element {
    table(RowSelection::none())
}

#[allow(non_snake_case)]
fn TableWithFirst() -> Element {
    table(RowSelection::single(0))
}

fn grid(start: RowSelection) -> Element {
    let mut chosen = use_signal(|| start);
    let items: Vec<CollectionItem<u32>> = (0..TILES)
        .map(|n| {
            CollectionItem::new(
                n,
                format!("File {n}"),
                Thumbnail::Icon(IconSource::Glyph(Icon::File)),
            )
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:300px; width:400px; display:flex; flex-direction:column",
                CollectionView::<u32> {
                    label: "Files",
                    items,
                    flow: Flow::Columns(COLUMNS),
                    selection: Some(chosen()),
                    onmarquee: move |next: RowSelection| chosen.set(next),
                }
            }
            p { class: "chosen", "{printed(&chosen())}" }
        }
    }
}

#[allow(non_snake_case)]
fn EmptyGrid() -> Element {
    grid(RowSelection::none())
}

fn harness(app: fn() -> Element) -> Harness {
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn chosen(harness: &Harness) -> String {
    harness.text_of(".chosen").unwrap_or_default()
}

fn row(index: usize) -> String {
    format!(".ds-vtable-item[data-row=\"{index}\"]")
}

fn tile(index: u32) -> String {
    format!(".ds-cv-item[data-index=\"{index}\"]")
}

/// A point in the empty space under the last row, in the first column.
fn under_the_rows(harness: &Harness) -> Point {
    let last = harness.rect(&row(ROWS - 1)).expect("the last row");
    Point {
        x: Px(last.left().0 + 40.0),
        y: Px(last.bottom().0 + 30.0),
    }
}

fn drag_holding(harness: &mut Harness, from: Point, to: Point, held: Modifiers) {
    let drag = PointerInput::new(from, PointerAction::Drag { to, steps: 4 });
    harness.send(Input::Pointer(drag.with_mods(held)));
}

#[test]
fn a_drag_from_under_the_rows_selects_the_rows_it_crosses() {
    let mut harness = harness(EmptyTable);
    let from = under_the_rows(&harness);
    let to = Point {
        x: from.x,
        y: harness.centre(&row(1)).expect("row 1").y,
    };
    harness.send(Input::drag(from, to, 4));
    assert_eq!(chosen(&harness), "1,2");
}

#[test]
fn a_click_on_empty_space_clears_the_selection() {
    let mut harness = harness(TableWithFirst);
    assert_eq!(chosen(&harness), "0");
    let at = under_the_rows(&harness);
    harness.send(Input::click(at));
    assert_eq!(chosen(&harness), "");
}

#[test]
fn shift_adds_the_rows_to_the_selection_the_drag_began_with() {
    let mut harness = harness(TableWithFirst);
    let from = under_the_rows(&harness);
    let to = Point {
        x: from.x,
        y: harness.centre(&row(2)).expect("row 2").y,
    };
    drag_holding(&mut harness, from, to, Modifiers::SHIFT);
    assert_eq!(chosen(&harness), "0,2");
}

#[test]
fn a_drag_that_begins_on_a_row_is_not_a_band() {
    let mut harness = harness(TableWithFirst);
    let from = harness.centre(&row(1)).expect("row 1");
    let to = harness.centre(&row(2)).expect("row 2");
    harness.send(Input::drag(from, to, 4));
    assert_eq!(chosen(&harness), "0");
}

#[test]
fn a_drag_from_under_the_tiles_selects_the_tiles_inside_the_band() {
    let mut harness = harness(EmptyGrid);
    let last = harness.rect(&tile(TILES - 1)).expect("the last tile");
    let first_column = harness.centre(&tile(4)).expect("tile 4");
    let second_column = harness.centre(&tile(5)).expect("tile 5");
    let from = Point {
        x: first_column.x,
        y: Px(last.bottom().0 + 30.0),
    };
    let to = Point {
        x: second_column.x,
        y: second_column.y,
    };
    harness.send(Input::drag(from, to, 4));
    assert_eq!(chosen(&harness), "4,5");
}

#[test]
fn a_press_on_a_tile_is_not_a_band_and_a_click_on_empty_space_clears() {
    let mut harness = harness(EmptyGrid);
    let last = harness.rect(&tile(TILES - 1)).expect("the last tile");
    let under = Point {
        x: Px(last.left().0 + 10.0),
        y: Px(last.bottom().0 + 30.0),
    };
    let from = harness.centre(&tile(0)).expect("tile 0");
    let to = harness.centre(&tile(5)).expect("tile 5");
    harness.send(Input::drag(from, to, 4));
    assert_eq!(chosen(&harness), "", "a drag from a tile selects nothing");
    harness.send(Input::drag(under, from, 4));
    assert_eq!(chosen(&harness), "0,1,2,3,4,5,6,7");
    harness.send(Input::click(under));
    assert_eq!(chosen(&harness), "");
}
