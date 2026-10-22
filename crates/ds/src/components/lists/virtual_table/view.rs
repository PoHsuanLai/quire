//! VirtualTable: `Table`'s header and columns over any number of rows that are never built. The
//! caller says how many rows there are and answers `cell(row, column)` for the ones that show: the
//! rows in the viewport and `overscan` more on each side are asked for, mounted as `Row`s of one
//! height (`row_pitch`, the compact row's token), and the rest of the table is two spacers' height
//! inside a `Scroller`, so the data (a million lines of a CSV file) stays wherever the caller keeps
//! it. The header sits above the scroller, so it stays while the rows scroll.
//!
//! A row's identity is its index: the cursor, the selection and `onselect` are row indexes. The
//! cursor is the row the keys rest on; the owner moves it (a controlled component, as
//! `VirtualList`'s), and it is kept in view. Scrolling to a row from outside is the owner's
//! `ScrollerRef`: `scroller.reveal_row(row, row_pitch())`. The arrows, Home, End, Page Up and
//! Page Down ask `onselect` to move the cursor and stop at the ends; Enter and Space ask
//! `onpick`.
//!
//! `onopen` hears the row to open (Finder's Open): a double click on a row, and Command-Down (the
//! primary modifier with Down) on the cursor's row. Return stays `onpick`'s, as in Finder, where
//! it renames and does not open. Command-Down is Finder's Open and is taken only when the owner gave `onopen`; it is then consumed, so the
//! app's keymap does not open the row a second time. Every other key with a modifier besides
//! Shift is left alone and goes on to the app's keymap (Command-Up is Enclosing Folder, so the
//! table must not move its cursor on it). Plain arrows move the cursor and Shift with an arrow
//! extends the selection, which `onreach` reports as `PickHow::Extend`; a click reports
//! `Replace`, `Toggle` or `Extend` the same way, so an owner never reads a modifier.
//!
//! Which rows are selected is the cursor's row alone, unless the owner gives a `selection` (a
//! `RowSelection`, ranges and never a key per row, so a million selected rows cost the same as
//! one). Then a row is selected when `selection.contains(row)`. The owner who wants several rows
//! listens to `onreach` instead of `onselect`: it hears a `Pick`, the row and how it was reached,
//! that the owner folds into its `RowSelection` with `RowSelection::picked`. A plain click or
//! arrow replaces the selection, the primary modifier (Command, or Ctrl) toggles the row, and
//! Shift (with a click or an arrow) extends from the anchor. Select all (Command A) and Copy are
//! the keymap's, not the table's: the owner answers them with `RowSelection::all`.
//!
//! `pinned` holds that many leading columns in place, header and cells, while the other columns
//! scroll sideways under them (Numbers' frozen columns, Finder's pinned Name). It cannot be
//! `position: sticky`, which Blitz does not lay out, so the table is two panes, built in
//! `split.rs`, whose doc comment says how they stay one table. A table that holds no column is
//! one pane whose root scrolls sideways, as before.
//!
//! Markup: `div.ds-table.ds-vtable.ds-list[role=group]` (the grid template as `--table-cols`) of
//! `div.ds-vtable-body` of the `Table`'s `div.ds-table-header` and a `Scroller` of a spacer,
//! `div.ds-vtable-rows[role=listbox]` of `div.ds-vtable-item` (one `Row` each, `data-row` its
//! index; `data-selected=yes` on the cursor's, `data-stripe=odd` on a striped one) and a spacer.
//! `data-look=striped|unstriped` is a list look (absent for the plain grid), `data-rules=hairline`
//! draws the lines between columns and `data-scrolled=yes|no` (unless the header rule is `Never`)
//! keys the line under the header. A table wider than its box scrolls sideways, header and rows
//! together. With `pinned` columns the root carries `data-pinned=yes` and `data-across=yes|no`
//! (the other columns are scrolled sideways or not), and the body is `split::body`'s: see there.
//! `columns_id(id)` names the box that scrolls sideways then.
//!
//! With `outline` the first cell is `span.ds-vtable-outline` (its `padding-left` the depth's
//! indent) of a `span.ds-vtable-twisty[role=button]` (or `span.ds-vtable-twisty-gap` on a leaf) and
//! `span.ds-vtable-outline-cell`, and the item carries `aria-level` and, on a branch,
//! `aria-expanded`. While a rubber band is dragged a `div.ds-marquee` is the scroller's first
//! child, placed in the content's coordinates.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::marquee::band::MarqueeBand;
use crate::components::lists::marquee::hook::use_marquee;
use crate::components::lists::marquee::model::rows_in;
use crate::components::lists::reach::Reach;
use crate::components::lists::row::row::Row;
use crate::components::lists::table::head::{cells, use_resize};
use crate::components::lists::table::look::TableLook;
use crate::components::lists::table::model::{Sort, TableColumn};
use crate::components::lists::table::rules::{ColumnRules, HeaderRule, Scrolled};
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::view::use_cursor_reveal;
use crate::components::lists::virtual_table::model::{
    TableKey, held_columns, key_how, page_rows, table_key, target,
};
use crate::components::lists::virtual_table::outline::{
    OutlineKind, OutlineRow, OutlineToggle, outline_cell,
};
use crate::components::lists::virtual_table::split::{self, Line, Pane, Panes};
use crate::keys::use_platform;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::press::Press;
use ds_core::selection::{Pick, RowSelection};
use ds_core::vocab::{RowState, Selection};
use ds_core::word::Word;
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

/// One row's content: its cells on the grid of `columns`, selected or not.
fn row_content<C>(
    columns: &[TableColumn<C>],
    drawn: &[Element],
    row: usize,
    selection: Selection,
    reach: Reach,
) -> Element {
    rsx! {
        Row {
            state: RowState {
                selection,
                ..RowState::default()
            },
            content: Some(cells(columns, drawn)),
            title: format!("Row {}", row + 1),
            onclick: move |press: Press| reach.clicked(row, press),
        }
    }
}

/// A table of `rows` rows in `columns`. `cell` is asked for `(row, column)` of each row that
/// shows, and must be cheap: it is called on every render of the table for every mounted row.
/// `scroller` is the table's scroll state, made by the owner with `use_scroller()` so it can scroll
/// to a row. `sort` is the sorted column and `on_sort` hears what a header press asks for.
/// `cursor` is the row the keys rest on, kept in view; `onselect` hears a click, an arrow, Home,
/// End, Page Up or Page Down asking for another row, and `onpick` hears Enter or Space on the
/// cursor. `onopen` hears the index of the row to open: a double click on it, or
/// Command-Down on the cursor's row. `selection`, when given, says which rows are selected (the cursor's row alone
/// otherwise), and `onreach` hears the same asks with how the row was reached (`Pick`) in place
/// of `onselect`: Shift with a click or an arrow extends, the primary modifier with a click
/// toggles. `pinned` is how many leading columns are held in place while the others scroll
/// sideways under them (none by default; at least one column always scrolls).
/// A drag that begins on empty space below the rows draws a rubber band in the accent tint and
/// selects the rows it touches; `onmarquee` hears the whole selection (a `RowSelection`, to store
/// as the owner's `selection`) as the band changes it. With no modifier the band's rows are the
/// selection and a click on empty space clears it; Shift or the primary modifier adds them to
/// the selection the drag began with (`selection`, or the cursor's row). The band scrolls the
/// table when the pointer moves near the top or bottom edge. Without `onmarquee` such a drag does
/// nothing.
///
/// `outline`, given, makes the first column an outline (Finder's list view): it is asked for each
/// mounted row's depth and whether it is a branch, open or closed ([`OutlineRow`]), and the first
/// cell is drawn indented with a disclosure triangle. A press on the triangle, and Right or Left
/// on the cursor's row, ask `ontoggle` for an [`OutlineToggle`]; with Option held the toggle
/// reaches the whole subtree. The table keeps no open state: the app lists the rows that show and
/// remembers which branches are open.
///
/// `overscan` is how many rows are mounted beyond the visible ones on each side (4 by default).
/// `look` dresses the table (`TableLook::Plain`, the grid, by default; `Striped` and `Unstriped`
/// are Finder's list view). `rules` draws a hairline between the columns; `header_rule` draws one under the
/// header once the rows are scrolled (`HeaderRule::WhenScrolled`, the default). `pitch` is how
/// tall every row is ([`row_pitch`], the compact row, by default): a table that zooms its rows
/// gives the zoomed height, and a row may be shorter than the compact row.
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
    #[props(default)] selection: Option<RowSelection>,
    #[props(default)] onselect: Option<EventHandler<usize>>,
    #[props(default)] onreach: Option<EventHandler<Pick>>,
    #[props(default)] onpick: Option<EventHandler<usize>>,
    #[props(default)] onopen: Option<EventHandler<usize>>,
    #[props(default)] onmarquee: Option<EventHandler<RowSelection>>,
    #[props(default)] outline: Option<Callback<usize, OutlineRow>>,
    #[props(default)] ontoggle: Option<EventHandler<OutlineToggle>>,
    #[props(default)] pinned: usize,
    #[props(default = OVERSCAN_ROWS)] overscan: usize,
    #[props(default)] look: TableLook,
    #[props(default)] rules: ColumnRules,
    #[props(default)] header_rule: HeaderRule,
    #[props(default = row_pitch())] pitch: Px,
    #[props(default)] common: Common,
) -> Element {
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
    let marquee = use_marquee(scroller);
    let mut held = use_signal(|| Held::No);
    let hovered = use_signal(|| None::<usize>);
    let across = use_signal(|| Px(0.0));
    let reach = Reach {
        platform: use_platform(),
        onreach,
        onselect,
        onopen,
    };
    let kept = held_columns(pinned, columns.len());
    let selected = |row: usize| {
        let on = match &selection {
            Some(chosen) => chosen.contains(row),
            None => cursor == Some(row),
        };
        match on {
            true => Selection::Selected,
            false => Selection::Unselected,
        }
    };
    let steps: Vec<Option<OutlineRow>> = (start..end)
        .map(|row| outline.map(|outline| outline.call(row)))
        .collect();
    let drawn_rows: Vec<(usize, Vec<Element>)> = (start..end)
        .zip(&steps)
        .map(|(row, step)| {
            let drawn = (0..columns.len())
                .map(|column| {
                    let content = cell.call((row, column));
                    match (column, step) {
                        (0, Some(step)) => outline_cell(*step, row, ontoggle, content),
                        _ => content,
                    }
                })
                .collect();
            (row, drawn)
        })
        .collect();
    let step_of = |row: usize| steps.get(row - start).copied().flatten();
    let onkeydown = move |event: KeyboardEvent| {
        let Some(act) = table_key(&event.key(), event.modifiers(), reach.platform) else {
            return;
        };
        let resting = cursor.filter(|at| *at < rows);
        match act {
            TableKey::Pick => {
                if let (Some(at), Some(onpick)) = (resting, onpick) {
                    event.prevent_default();
                    onpick.call(at);
                }
            }
            TableKey::Open => {
                if let (Some(at), Some(_)) = (resting, onopen) {
                    event.prevent_default();
                    event.stop_propagation();
                    reach.opened(at);
                }
            }
            TableKey::Disclose { to, reach } => {
                let branch = resting.and_then(|at| outline.map(|outline| (at, outline.call(at))));
                if let (Some((row, step)), Some(ontoggle)) = (branch, ontoggle)
                    && step.kind == OutlineKind::Branch(to.flipped())
                {
                    event.prevent_default();
                    ontoggle.call(OutlineToggle {
                        row,
                        shown: to,
                        reach,
                    });
                }
            }
            TableKey::Move(_) | TableKey::Page(_) => {
                let shown = scroller.scroll().read().viewport.0;
                let page = page_rows(shown, pitch.0);
                if let (Some(row), true) = (target(cursor, rows, act, page), reach.heard()) {
                    event.prevent_default();
                    reach.tell(Pick {
                        row,
                        how: key_how(event.modifiers()),
                    });
                }
            }
        }
    };
    let move_columns = columns.clone();
    let began_with = selection
        .clone()
        .unwrap_or_else(|| cursor.map_or_else(RowSelection::none, RowSelection::single));
    let data = common.data_attributes();
    let mounted = common.clone();
    let away = (held() == Held::No).then_some("away");
    let scrolled = header_rule.attribute(scroller.scroll().read().offset);
    let (above, below) = (layout.top(start).0, layout.below(end).0);
    let height = pitch.0;
    let frame = match kept {
        0 => {
            let header = resize.header(&columns, sort.as_ref(), on_sort);
            let template = resize.template(&columns);
            let wide = resize.total(&columns).0;
            let items: Vec<(usize, Element, Option<OutlineRow>)> = drawn_rows
                .iter()
                .map(|(row, drawn)| {
                    let content = row_content(&columns, drawn, *row, selected(*row), reach);
                    (*row, content, step_of(*row))
                })
                .collect();
            let body = rsx! {
                div { class: "ds-vtable-body", style: "min-width:{wide}px",
                    {header}
                    Scroller { scroller, common: marquee.common(),
                        MarqueeBand { marquee }
                        div { class: "ds-vtable-spacer", style: "height:{above}px" }
                        div {
                            class: "ds-vtable-rows",
                            role: "listbox",
                            "aria-label": label.clone(),
                            for (row , content , step) in items {
                                div {
                                    key: "{row}",
                                    class: "ds-vtable-item",
                                    role: "none",
                                    "aria-posinset": (row + 1).to_string(),
                                    "aria-setsize": rows.to_string(),
                                    "aria-level": step.map(|step| step.level()),
                                    "aria-expanded": step.and_then(OutlineRow::expanded),
                                    "data-row": row.to_string(),
                                    "data-selected": (selected(row) == Selection::Selected).then_some("yes"),
                                    "data-stripe": look.striped(row).then_some("odd"),
                                    style: "height:{height}px",
                                    onpointerdown: move |_| marquee.item_pressed(),
                                    ondoubleclick: move |_| reach.opened(row),
                                    {content}
                                }
                            }
                        }
                        div { class: "ds-vtable-spacer", style: "height:{below}px" }
                    }
                }
            };
            (body, Some(format!("--table-cols:{template}")), None, None)
        }
        kept => {
            let all = columns.len();
            let lines = |span: std::ops::Range<usize>| -> Vec<Line> {
                drawn_rows
                    .iter()
                    .map(|(row, drawn)| Line {
                        row: *row,
                        selection: selected(*row),
                        stripe: look.striped(*row).then_some("odd"),
                        outline: step_of(*row),
                        content: row_content(
                            &columns[span.clone()],
                            &drawn[span.clone()],
                            *row,
                            selected(*row),
                            reach,
                        ),
                    })
                    .collect()
            };
            let body = split::body(Panes {
                held: Pane {
                    head: resize.header_in(&columns, 0..kept, sort.as_ref(), on_sort),
                    lines: lines(0..kept),
                    template: resize.template_in(&columns, 0..kept),
                    width: resize.total_in(&columns, 0..kept),
                },
                scrolling: Pane {
                    head: resize.header_in(&columns, kept..all, sort.as_ref(), on_sort),
                    lines: lines(kept..all),
                    template: resize.template_in(&columns, kept..all),
                    width: resize.total_in(&columns, kept..all),
                },
                scroller,
                label: label.clone(),
                id: common.id.clone(),
                rows,
                above,
                below,
                height,
                across,
                hovered,
                onopen,
                marquee,
            });
            let ahead = Some(Scrolled::at(across()).slug());
            (body, None, Some("yes"), ahead)
        }
    };
    let (body, inline, pin, across_mark) = frame;
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-table ds-vtable ds-list"),
            role: "group",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "data-focus": away,
            "data-look": look.attribute(),
            "data-rules": rules.attribute(),
            "data-scrolled": scrolled,
            "data-pinned": pin,
            "data-across": across_mark,
            style: inline,
            onmounted: move |event| mounted.mounted(event),
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown,
            onpointerdown: move |event: PointerEvent| {
                marquee.pressed(&event, began_with.clone(), onmarquee);
            },
            onpointermove: move |event: PointerEvent| {
                resize.drag(&move_columns, &event);
                marquee.moved(&event, &|band| vec![rows_in(band, pitch, rows)], onmarquee);
            },
            onpointerup: move |_| {
                resize.release();
                marquee.end();
            },
            onpointerleave: move |_| {
                resize.release();
                marquee.end();
            },
            ..data,
            {body}
        }
    }
}
