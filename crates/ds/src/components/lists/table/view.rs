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
//! `div.ds-table-cells` of `span.ds-table-cell`.

use crate::components::controls::edge_grab::EdgeGrab;
use crate::components::lists::list::list::List;
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::components::lists::row::row::Row;
use crate::components::lists::row::size::RowSize;
use crate::components::lists::table::model::{
    CellAlign, Sort, SortDirection, Sorting, TableColumn, TableRow,
};
use crate::components::lists::table::widths::Widths;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::{RowState, Selection};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use std::hash::Hash;

/// The word `aria-sort` takes for a column.
fn aria_sort(sort: Option<&Sort<impl PartialEq>>, is: bool) -> &'static str {
    match (sort, is) {
        (Some(sort), true) => match sort.direction {
            SortDirection::Ascending => "ascending",
            SortDirection::Descending => "descending",
        },
        _ => "none",
    }
}

/// A table of `rows` in `columns`. `sort` is the sorted column; `on_sort` hears what a header
/// press asks for. `selection` are the selected rows and `cursor` the one the keys rest on;
/// `onselect` hears a click, an arrow or a typed letter reaching a row. `size` is the rows'
/// height (24 by default).
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
    #[props(default)] common: Common,
) -> Element {
    let mut widths = use_signal(Widths::default);
    let mut grab = use_signal(|| None::<(usize, EdgeGrab)>);
    let template = widths.read().template(&columns);
    let heads = columns.iter().enumerate().map(|(at, column)| {
        let sorted = sort.as_ref().is_some_and(|sort| sort.column == column.id);
        let width = widths.read().of(&columns, at);
        let last = at + 1 == columns.len();
        header_cell(
            column,
            aria_sort(sort.as_ref(), sorted),
            sort.as_ref(),
            on_sort,
            sorted,
        )
        .map(|head| (head, at, width, last))
    });
    let heads: Vec<_> = heads.flatten().collect();
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
            style: "--table-cols:{template}",
            onmounted: move |event| common.mounted(event),
            onpointermove: move |event: PointerEvent| {
                if let Some((at, held)) = grab() {
                    let width = held.size_at(Px(event.client_coordinates().x as f32));
                    let next = widths.peek().clone().dragged(&move_columns, at, width);
                    widths.set(next);
                }
            },
            onpointerup: move |_| grab.set(None),
            onpointerleave: move |_| grab.set(None),
            ..data,
            div { class: "ds-table-header", role: "row",
                for (head , at , width , last) in heads {
                    div { key: "{at}", class: "ds-table-head",
                        {head}
                        if !last {
                            span {
                                class: "ds-table-resize",
                                "aria-hidden": "true",
                                onpointerdown: move |event: PointerEvent| {
                                    event.stop_propagation();
                                    let from = Px(event.client_coordinates().x as f32);
                                    grab.set(Some((at, EdgeGrab::new(from, width))));
                                },
                            }
                        }
                    }
                }
            }
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

/// One header: the title, and for a sortable column the button and the indicator.
fn header_cell<C: Clone + PartialEq + 'static>(
    column: &TableColumn<C>,
    aria: &'static str,
    sort: Option<&Sort<C>>,
    on_sort: EventHandler<Sort<C>>,
    sorted: bool,
) -> Option<Element> {
    let id = column.id.clone();
    let asked = Sort::after_press(sort, &id);
    let arrow = sort.filter(|_| sorted).map(|sort| match sort.direction {
        SortDirection::Ascending => Icon::ChevronUp,
        SortDirection::Descending => Icon::ChevronDown,
    });
    let live = column.sorting == Sorting::Sortable;
    Some(rsx! {
        button {
            r#type: "button",
            class: "ds-table-column",
            role: "columnheader",
            "aria-sort": aria,
            "data-align": column.align.slug(),
            "data-sorting": column.sorting.slug(),
            "aria-disabled": if live { None } else { Some("true") },
            onclick: move |_| {
                if live {
                    on_sort.call(asked.clone());
                }
            },
            span { class: "ds-table-title", "{column.title}" }
            if let Some(icon) = arrow {
                span { class: "ds-table-sort", "aria-hidden": "true",
                    Glyph { icon, size: IconSize::Micro }
                }
            }
        }
    })
}

/// A row's cells laid on the columns' grid.
fn cells<C>(columns: &[TableColumn<C>], cells: &[Element]) -> Element {
    rsx! {
        span { class: "ds-table-cells",
            for (at , cell) in cells.iter().enumerate() {
                span {
                    key: "{at}",
                    class: "ds-table-cell ds-truncate",
                    "data-align": columns.get(at).map_or(CellAlign::Leading, |column| column.align).slug(),
                    {cell.clone()}
                }
            }
        }
    }
}
