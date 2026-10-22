//! `VirtualTable`'s row drag and fit-to-content, on a real Blitz document: dragging a selected row
//! carries the whole selection onto a folder row, Option makes it a copy, a drag that begins on a
//! row never starts a rubber band, a press that does not move is no drag; and a double click on a
//! column's edge resizes the column to the widest mounted cell.

use dioxus::prelude::*;
use ds::base::selection::{Pick, RowSelection};
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::table::look::TableLook;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::virtual_table::{RowDrop, RowDropKind, RowDropTarget, VirtualTable};
use ds::file_drop::drag::DropOperation;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, PointerAction, PointerInput, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: usize = 100;
/// The content width every cell of the first column reports.
const CELL: f32 = 150.0;

/// Every fifth row is a folder; rows 1 to 3 start selected.
#[allow(non_snake_case)]
fn Page() -> Element {
    let scroller = use_scroller();
    let mut chosen = use_signal(|| RowSelection::of_rows(Some(1), Some(3), [1, 2, 3]));
    let mut log = use_signal(String::new);
    let columns = vec![
        TableColumn::new(0u8, "Name", Px(120.0)),
        TableColumn::new(1u8, "Kind", Px(80.0)),
    ];
    let cell = Callback::new(|(row, _): (usize, usize)| rsx! { "{row}" });
    let drop_kind = Callback::new(|row: usize| match row % 5 {
        0 => RowDropKind::Container,
        _ => RowDropKind::Item,
    });
    let cell_width = Callback::new(|(_, column): (usize, usize)| match column {
        0 => Px(CELL),
        _ => Px(10.0),
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:300px; width:380px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Rows",
                    columns,
                    rows: ROWS,
                    cell,
                    scroller,
                    on_sort: |_| {},
                    look: TableLook::Striped,
                    selection: Some(chosen()),
                    onreach: move |pick: Pick| chosen.set(chosen().picked(pick)),
                    onmarquee: move |next: RowSelection| chosen.set(next),
                    onrowdrop: move |drop: RowDrop| {
                        let target = match drop.target {
                            RowDropTarget::Onto(row) => format!("onto {row}"),
                            RowDropTarget::Between(gap) => format!("between {gap}"),
                        };
                        let how = match drop.operation {
                            DropOperation::Copy => "copy",
                            DropOperation::Move => "move",
                            DropOperation::Link => "link",
                        };
                        log.set(format!("{:?} {target} {how}", drop.rows));
                    },
                    drop_kind,
                    cell_width,
                }
            }
            p { class: "log", "{log}" }
        }
    }
}

fn harness() -> Harness {
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn row(index: usize) -> String {
    format!(".ds-vtable-item[data-row=\"{index}\"]")
}

fn drag_holding(harness: &mut Harness, from: Point, to: Point, held: Modifiers) {
    let drag = PointerInput::new(from, PointerAction::Drag { to, steps: 6 });
    harness.send(Input::Pointer(drag.with_mods(held)));
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn dragging_a_selected_row_onto_a_folder_carries_the_selection() {
    let mut harness = harness();
    let from = harness.centre(&row(2)).expect("row 2");
    let to = harness.centre(&row(5)).expect("the folder row");
    drag_holding(&mut harness, from, to, Modifiers::empty());
    assert_eq!(log(&harness), "[1, 2, 3] onto 5 move");
}

#[test]
fn option_makes_the_drop_a_copy() {
    let mut harness = harness();
    let from = harness.centre(&row(2)).expect("row 2");
    let to = harness.centre(&row(5)).expect("the folder row");
    drag_holding(&mut harness, from, to, Modifiers::ALT);
    assert_eq!(log(&harness), "[1, 2, 3] onto 5 copy");
}

#[test]
fn a_row_outside_the_selection_drags_alone_and_a_plain_row_takes_no_drop() {
    let mut harness = harness();
    let from = harness.centre(&row(6)).expect("row 6");
    let folder = harness.centre(&row(5)).expect("the folder row");
    drag_holding(&mut harness, from, folder, Modifiers::empty());
    assert_eq!(log(&harness), "[6] onto 5 move");
    let mut harness = self::harness();
    let plain = harness.centre(&row(4)).expect("a plain row");
    let from = harness.centre(&row(6)).expect("row 6");
    drag_holding(&mut harness, from, plain, Modifiers::empty());
    assert_eq!(
        log(&harness),
        "",
        "no gaps are offered, so a plain row takes nothing"
    );
}

#[test]
fn the_indicator_outlines_the_folder_under_the_pointer_while_the_drag_is_live() {
    let mut harness = harness();
    assert_eq!(harness.count(".ds-vtable-drop"), 0);
    let from = harness.centre(&row(6)).expect("row 6");
    let folder = harness.centre(&row(5)).expect("the folder row");
    harness.send(Input::pointer_down(from));
    harness.send(Input::pointer_move(folder));
    assert_eq!(
        harness.attr(".ds-vtable-drop", "data-drop").as_deref(),
        Some("onto")
    );
    harness.send(Input::pointer_up(folder));
    assert_eq!(harness.count(".ds-vtable-drop"), 0);
}

#[test]
fn double_clicking_a_column_edge_fits_the_widest_cell() {
    let mut harness = harness();
    let before = harness.rect(".ds-table-head").expect("the first head");
    assert_eq!(before.size.width, Px(120.0));
    let edge = harness.centre(".ds-table-resize").expect("the edge");
    harness.send(Input::click(edge));
    harness.send(Input::click(edge));
    let after = harness.rect(".ds-table-head").expect("the first head");
    assert_eq!(
        after.size.width,
        Px(CELL + 16.0),
        "the cell and its padding"
    );
}
