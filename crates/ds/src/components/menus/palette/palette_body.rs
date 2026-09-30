//! Drawing the command palette's groups: each group's `SectionHeader` with its
//! action, then its rows as menu rows or its emoji grid, every row, cell and action reporting by
//! its stop number (`palette_stops`). Split from `command_palette`.

use crate::components::lists::emoji_grid::grid::{CellEvents, draw_cells, grid_style};
use crate::components::lists::row::row::{Row, RowMounted};
use crate::components::lists::row::size::RowSize;
use crate::components::lists::section_header::SectionHeader;
use crate::components::menus::palette::palette_motion::ListMotion;
use crate::components::menus::palette::palette_stops::{Body, ShownGroup};
use dioxus::prelude::*;
use ds_core::vocab::{RowState, Selection};
use std::cell::Cell;
use std::rc::Rc;

/// Whether the palette that draws the rows is still mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Life {
    Up,
    Gone,
}

/// What the drawn stops report, by stop number.
#[derive(Clone)]
pub(crate) struct StopEvents {
    /// Whether the palette is still up: a row mounts after it may be gone.
    pub alive: Rc<Cell<Life>>,
    /// A click on a row or a cell.
    pub pick: EventHandler<usize>,
    /// The pointer moved over a row or a cell.
    pub point: EventHandler<usize>,
    /// A row's or a cell's element mounted.
    pub mounted: EventHandler<(usize, MountedEvent)>,
    /// A header action's element mounted (it reports no rect, but is kept in view).
    pub action_mounted: EventHandler<(usize, MountedEvent)>,
    /// A header action was clicked (before it runs).
    pub action_ran: EventHandler<()>,
}

/// Every drawn group, the stop `current` highlighted, each group's rows playing their part in
/// `motion`.
pub(crate) fn draw_groups<T>(
    shown: &[ShownGroup<'_, T>],
    current: usize,
    events: StopEvents,
    motion: &ListMotion,
) -> Element {
    let drawn: Vec<Element> = shown
        .iter()
        .map(|group| draw_group(group, current, events.clone(), motion))
        .collect();
    rsx! {
        for (key , group) in drawn.into_iter().enumerate() {
            Fragment { key: "{key}", {group} }
        }
    }
}

/// One group: its header, then its rows or its grid.
fn draw_group<T>(
    shown: &ShownGroup<'_, T>,
    current: usize,
    events: StopEvents,
    motion: &ListMotion,
) -> Element {
    let first = shown.first;
    let local = current.checked_sub(first);
    let (body, size) = match &shown.body {
        Body::Rows(rows) => {
            let size = rows.len();
            let motion = motion.rows_of(&shown.group.title);
            let drawn: Vec<Element> = rows
                .iter()
                .enumerate()
                .map(|(at, marked)| {
                    let row = marked.row;
                    let state = RowState {
                        selection: Selection::of(&Some(at), &local),
                        availability: row.availability,
                        ..RowState::default()
                    };
                    rsx! {
                        Row {
                            key: "{at}",
                            leading: row.leading.clone(),
                            title: row.title.clone(),
                            marks: marked.marks.clone(),
                            detail: row.detail.clone(),
                            accessory: row.accessory.clone(),
                            chord: row.chord.clone(),
                            shape: row.shape.clone(),
                            action: row.action.clone(),
                            state,
                            size: RowSize::Settings,
                            motion: motion.of(at),
                            onclick: move |_| events.pick.call(first + at),
                            onpointermove: move |_| events.point.call(first + at),
                            onmounted: {
                                let alive = events.alive.clone();
                                RowMounted::new(move |event: MountedEvent| {
                                    if alive.get() == Life::Up {
                                        events.mounted.call((first + at, event));
                                    }
                                })
                            },
                        }
                    }
                })
                .collect();
            (rsx! { {drawn.into_iter()} }, size)
        }
        Body::Grid(grid) => {
            let cells = draw_cells(
                &grid.cells,
                local,
                CellEvents {
                    pick: EventHandler::new(move |at: usize| events.pick.call(first + at)),
                    point: EventHandler::new(move |at: usize| events.point.call(first + at)),
                    mounted: Some(EventHandler::new(
                        move |(at, event): (usize, MountedEvent)| {
                            events.mounted.call((first + at, event))
                        },
                    )),
                },
            );
            let body = rsx! {
                div {
                    class: "ds-emoji-grid ds-emoji-text",
                    role: "grid",
                    "aria-label": "{shown.group.title}",
                    style: grid_style(grid.columns, grid.cell),
                    {cells}
                }
            };
            (body, grid.cells.len())
        }
    };
    let action_selection = Selection::of(&Some(first + size), &Some(current));
    rsx! {
        SectionHeader {
            title: shown.group.title.clone(),
            action: shown.group.action.clone().map(|(label, run)| {
                let booked = EventHandler::new(move |()| {
                    events.action_ran.call(());
                    run.call(());
                });
                (label, booked)
            }),
            action_selection: match shown.group.action {
                Some(_) => action_selection,
                None => Selection::Unselected,
            },
            on_action_mounted: EventHandler::new(move |event: MountedEvent| {
                events.action_mounted.call((first + size, event))
            }),
        }
        {body}
    }
}
