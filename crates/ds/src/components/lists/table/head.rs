//! A table's header and the column widths a person drags, shared by `Table` and `VirtualTable`:
//! the sortable titles, the sorted column's indicator and `aria-sort`, and the edge between two
//! headers that resizes.

use crate::components::controls::edge_grab::EdgeGrab;
use crate::components::lists::table::model::{
    CellAlign, Sort, SortDirection, Sorting, TableColumn,
};
use crate::components::lists::table::widths::Widths;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// The widths a table's columns have been dragged to, and the drag in progress. `Copy`, so the
/// table's handlers and the header's edges share it.
#[derive(Clone, Copy)]
pub(crate) struct Resize {
    widths: Signal<Widths>,
    grab: Signal<Option<(usize, EdgeGrab)>>,
}

/// The calling table's column widths.
pub(crate) fn use_resize() -> Resize {
    Resize {
        widths: use_signal(Widths::default),
        grab: use_signal(|| None::<(usize, EdgeGrab)>),
    }
}

impl Resize {
    /// `grid-template-columns` for `columns`.
    pub(crate) fn template<C>(&self, columns: &[TableColumn<C>]) -> String {
        self.widths.read().template(columns)
    }

    /// How wide `columns` are laid out together, the last at its own width.
    pub(crate) fn total<C>(&self, columns: &[TableColumn<C>]) -> Px {
        self.widths.read().total(columns)
    }

    /// The pointer moved: a column edge being dragged follows it.
    pub(crate) fn drag<C>(&self, columns: &[TableColumn<C>], event: &PointerEvent) {
        if let Some((at, held)) = (self.grab)() {
            let width = held.size_at(Px(event.client_coordinates().x as f32));
            let next = self.widths.peek().clone().dragged(columns, at, width);
            let mut widths = self.widths;
            widths.set(next);
        }
    }

    /// The pointer was released or left: the drag ends.
    pub(crate) fn release(&self) {
        let mut grab = self.grab;
        grab.set(None);
    }

    /// The header row: a title for each of `columns`, and a resize edge after each but the last.
    pub(crate) fn header<C: Clone + PartialEq + 'static>(
        &self,
        columns: &[TableColumn<C>],
        sort: Option<&Sort<C>>,
        on_sort: EventHandler<Sort<C>>,
    ) -> Element {
        let heads: Vec<(Element, usize, Px, bool)> = columns
            .iter()
            .enumerate()
            .map(|(at, column)| {
                let sorted = sort.is_some_and(|sort| sort.column == column.id);
                let width = self.widths.read().of(columns, at);
                let last = at + 1 == columns.len();
                (
                    header_cell(column, aria_sort(sort, sorted), sort, on_sort, sorted),
                    at,
                    width,
                    last,
                )
            })
            .collect();
        let grab = self.grab;
        rsx! {
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
                                    let mut grab = grab;
                                    grab.set(Some((at, EdgeGrab::new(from, width))));
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

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

/// One header: the title, and for a sortable column the button and the indicator.
fn header_cell<C: Clone + PartialEq + 'static>(
    column: &TableColumn<C>,
    aria: &'static str,
    sort: Option<&Sort<C>>,
    on_sort: EventHandler<Sort<C>>,
    sorted: bool,
) -> Element {
    let id = column.id.clone();
    let asked = Sort::after_press(sort, &id);
    let arrow = sort.filter(|_| sorted).map(|sort| match sort.direction {
        SortDirection::Ascending => Icon::ChevronUp,
        SortDirection::Descending => Icon::ChevronDown,
    });
    let live = column.sorting == Sorting::Sortable;
    rsx! {
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
    }
}

/// A row's cells laid on the columns' grid.
pub(crate) fn cells<C>(columns: &[TableColumn<C>], cells: &[Element]) -> Element {
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
