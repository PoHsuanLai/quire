//! VirtualTable: `Table`'s header and columns over any number of rows that are never built. The
//! caller says how many rows there are and answers `cell(row, column)` for the ones that show: the
//! rows in the viewport and `overscan` more on each side are asked for, mounted as `Row`s of one
//! height (`row_pitch`, the compact row's token), and the rest of the table is two spacers' height
//! inside a `Scroller`, so the data (a million lines of a CSV file) stays wherever the caller keeps
//! it. The header sits above the scroller, so it stays while the rows scroll.
//!
//! A row's identity is its index: the cursor, the selection and `onselect` are row indexes. The
//! cursor is the selected row; the owner moves it (a controlled component, as `VirtualList`'s), and
//! it is kept in view. Scrolling to a row from outside is the owner's `ScrollerRef`:
//! `scroller.reveal_row(row, row_pitch())`. The arrows, Home, End, Page Up and Page Down ask
//! `onselect` to move the cursor and stop at the ends; Enter and Space ask `onpick`.
//!
//! Markup: `div.ds-table.ds-vtable.ds-list[role=group]` (the grid template as `--table-cols`) of
//! `div.ds-vtable-body` of the `Table`'s `div.ds-table-header` and a `Scroller` of a spacer,
//! `div.ds-vtable-rows[role=listbox]` of `div.ds-vtable-item` (one `Row` each, `data-row` its
//! index) and a spacer. A table wider than its box scrolls sideways, header and rows together.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::row::row::Row;
use crate::components::lists::table::head::{cells, use_resize};
use crate::components::lists::table::model::{Sort, TableColumn};
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::view::use_cursor_reveal;
use crate::components::lists::virtual_table::model::{TableKey, page_rows, table_key, target};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::{RowState, Selection};
use ds_style::tokens::row_scale::ROW_SCALE;

/// How many rows around the viewport are mounted besides the visible ones.
const OVERSCAN_ROWS: usize = 4;

/// Whether the table holds the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Yes,
    No,
}

/// How tall every row of a `VirtualTable` is: the compact row's token (`--row-compact-h`), in
/// logical pixels. A row's top is `row * row_pitch()`, which is what a `ScrollerRef` needs to
/// scroll to it.
pub const fn row_pitch() -> Px {
    Px(ROW_SCALE.compact_height.0 as f32)
}

/// A table of `rows` rows in `columns`. `cell` is asked for `(row, column)` of each row that
/// shows, and must be cheap: it is called on every render of the table for every mounted row.
/// `scroller` is the table's scroll state, made by the owner with `use_scroller()` so it can scroll
/// to a row. `sort` is the sorted column and `on_sort` hears what a header press asks for.
/// `cursor` is the selected row, kept in view; `onselect` hears a click, an arrow, Home, End,
/// Page Up or Page Down asking for another row, and `onpick` hears Enter or Space on the cursor.
/// `overscan` is how many rows are mounted beyond the visible ones on each side (4 by default).
///
/// The row index and the pixel offset are `f32`s in the end: past about 500,000 rows (16 million
/// pixels) a row's top is no longer exact, so a table of more rows than that should be paged by
/// its owner.
#[component]
pub fn VirtualTable<C: Clone + PartialEq + 'static>(
    label: String,
    columns: Vec<TableColumn<C>>,
    rows: usize,
    cell: Callback<(usize, usize), Element>,
    scroller: ScrollerRef,
    #[props(default)] sort: Option<Sort<C>>,
    on_sort: EventHandler<Sort<C>>,
    #[props(default)] cursor: Option<usize>,
    #[props(default)] onselect: Option<EventHandler<usize>>,
    #[props(default)] onpick: Option<EventHandler<usize>>,
    #[props(default = OVERSCAN_ROWS)] overscan: usize,
    #[props(default)] common: Common,
) -> Element {
    let pitch = row_pitch();
    let layout = Layout::Even { pitch, len: rows };
    let window = use_memo(use_reactive(
        (&layout, &overscan),
        move |(layout, overscan)| layout.window(*scroller.scroll().read(), overscan),
    ));
    let range = window();
    let end = range.end.min(rows);
    let start = range.start.min(end);
    use_cursor_reveal(
        scroller,
        cursor.filter(|at| *at < rows).map(|at| layout.span(at)),
    );
    let resize = use_resize();
    let mut held = use_signal(|| Held::No);
    let template = resize.template(&columns);
    let total = resize.total(&columns);
    let header = resize.header(&columns, sort.as_ref(), on_sort);
    let items: Vec<(usize, Element)> = (start..end)
        .map(|row| {
            let drawn: Vec<Element> = (0..columns.len())
                .map(|column| cell.call((row, column)))
                .collect();
            let selected = cursor == Some(row);
            let content = rsx! {
                Row {
                    state: RowState {
                        selection: if selected { Selection::Selected } else { Selection::Unselected },
                        ..RowState::default()
                    },
                    content: Some(cells(&columns, &drawn)),
                    title: format!("Row {}", row + 1),
                    onclick: move |_| {
                        if let Some(onselect) = onselect {
                            onselect.call(row);
                        }
                    },
                }
            };
            (row, content)
        })
        .collect();
    let onkeydown = move |event: KeyboardEvent| {
        let Some(act) = table_key(&event.key(), event.modifiers()) else {
            return;
        };
        match act {
            TableKey::Pick => {
                if let (Some(at), Some(onpick)) = (cursor.filter(|at| *at < rows), onpick) {
                    event.prevent_default();
                    onpick.call(at);
                }
            }
            TableKey::Move(_) | TableKey::Page(_) => {
                let shown = scroller.scroll().read().viewport.0;
                let page = page_rows(shown, pitch.0);
                if let (Some(to), Some(onselect)) = (target(cursor, rows, act, page), onselect) {
                    event.prevent_default();
                    onselect.call(to);
                }
            }
        }
    };
    let move_columns = columns.clone();
    let data = common.data_attributes();
    let mounted = common.clone();
    let away = (held() == Held::No).then_some("away");
    let (above, below) = (layout.top(start).0, layout.below(end).0);
    let (height, wide) = (pitch.0, total.0);
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-table ds-vtable ds-list"),
            role: "group",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "data-focus": away,
            style: "--table-cols:{template}",
            onmounted: move |event| mounted.mounted(event),
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown,
            onpointermove: move |event: PointerEvent| resize.drag(&move_columns, &event),
            onpointerup: move |_| resize.release(),
            onpointerleave: move |_| resize.release(),
            ..data,
            div { class: "ds-vtable-body", style: "min-width:{wide}px",
                {header}
                Scroller { scroller,
                    div { class: "ds-vtable-spacer", style: "height:{above}px" }
                    div {
                        class: "ds-vtable-rows",
                        role: "listbox",
                        "aria-label": label.clone(),
                        for (row , content) in items {
                            div {
                                key: "{row}",
                                class: "ds-vtable-item",
                                role: "none",
                                "aria-posinset": (row + 1).to_string(),
                                "aria-setsize": rows.to_string(),
                                "data-row": row.to_string(),
                                style: "height:{height}px",
                                {content}
                            }
                        }
                    }
                    div { class: "ds-vtable-spacer", style: "height:{below}px" }
                }
            }
        }
    }
}
