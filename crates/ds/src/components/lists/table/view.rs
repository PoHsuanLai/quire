//! Table: a header of columns over a `List` of rows (design/30 section 2.6, `NSTableView`). A press
//! on a sortable header asks for a sort (`Sort::after_press`); the sorted header draws its
//! indicator and writes `aria-sort`; the edge between two headers drags 1:1, held at the column's
//! least, and the last column takes what is left. Rows are the list's: selection is the caller's,
//! a selected row is the accent under its ink while the table holds the keyboard and grey when it
//! does not, and no row is striped.
//!
//! Markup: `div.ds-table[role=group]` (the grid template as `--table-cols`) of
//! `div.ds-table-header` (`button.ds-table-column` with `.ds-table-title`, `.ds-table-sort`, and
//! a `span.ds-table-resize` after each but the last) and the `List`, whose rows hold
//! `div.ds-table-cells` of `span.ds-table-cell`. `data-rules=hairline` on the root draws the
//! lines between columns.

use crate::components::lists::list::list::List;
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::components::lists::row::row::Row;
use crate::components::lists::row::size::RowSize;
use crate::components::lists::table::head::{cells, use_resize};
use crate::components::lists::table::model::{Sort, TableColumn, TableRow};
use crate::components::lists::table::rules::ColumnRules;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{RowState, Selection};
use ds_core::word::Word;
use std::hash::Hash;

/// A table of `rows` in `columns`. `sort` is the sorted column; `on_sort` hears what a header
/// press asks for. `selection` are the selected rows and `cursor` the one the keys rest on;
/// `onselect` hears a click, an arrow or a typed letter reaching a row. `size` is the rows'
/// height (24 by default). `rules` draws a hairline between the columns.
#[component]
pub fn Table<K: Clone + PartialEq + Hash + 'static, C: Clone + PartialEq + 'static>(
    label: String,
    columns: Vec<TableColumn<C>>,
    rows: Vec<TableRow<K>>,
    #[props(default)] sort: Option<Sort<C>>,
    on_sort: EventHandler<Sort<C>>,
    #[props(default)] selection: Vec<K>,
    #[props(default)] cursor: Option<K>,
    onselect: EventHandler<K>,
    #[props(default)] size: RowSize,
    #[props(default)] rules: ColumnRules,
    #[props(default)] common: Common,
) -> Element {
    let resize = use_resize();
    let template = resize.template(&columns);
    let header = resize.header(&columns, sort.as_ref(), on_sort);
    let items: Vec<ListItem<K>> = rows
        .iter()
        .map(|row| {
            let key = row.key.clone();
            let selected = selection.contains(&row.key);
            ListItem::row(
                row.key.clone(),
                row.label.clone(),
                rsx! {
                    Row {
                        size,
                        state: RowState {
                            selection: if selected { Selection::Selected } else { Selection::Unselected },
                            availability: row.availability,
                            ..RowState::default()
                        },
                        content: Some(cells(&columns, &row.cells)),
                        title: row.label.clone(),
                        onclick: move |_| onselect.call(key.clone()),
                    }
                },
            )
            .with(row.availability)
        })
        .collect();
    let move_columns = columns.clone();
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-table"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "data-density": size.slug(),
            "data-rules": rules.attribute(),
            style: "--table-cols:{template}",
            onmounted: move |event| common.mounted(event),
            onpointermove: move |event: PointerEvent| resize.drag(&move_columns, &event),
            onpointerup: move |_| resize.release(),
            onpointerleave: move |_| resize.release(),
            ..data,
            {header}
            List::<K> {
                label: label.clone(),
                items,
                style: ListStyle::Plain,
                cursor,
                onselect: move |key| onselect.call(key),
            }
        }
    }
}
