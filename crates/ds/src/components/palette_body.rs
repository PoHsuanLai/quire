//! Drawing the command palette's groups (sill Q291, Q294): each group's `SectionHeader` with its
//! action, then its rows as menu rows or its emoji grid, every row, cell and action reporting by
//! its stop number (`palette_stops`). Split from `command_palette`.

use crate::components::emoji_grid::{CellEvents, draw_cells, grid_style};
use crate::components::menu::MenuKind;
use crate::components::menu_lines::choices_len;
use crate::components::menu_rows::{Drawn, render_lines};
use crate::components::palette_stops::{Body, ShownGroup};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::vocab::Selection;
use crate::geometry::Point;
use dioxus::prelude::*;

/// What the drawn stops report, by stop number.
#[derive(Clone, Copy)]
pub(crate) struct StopEvents {
    /// A click on a row or a cell.
    pub pick: EventHandler<usize>,
    /// The pointer moved over a row or a cell.
    pub point: EventHandler<usize>,
    /// A row's or a cell's element mounted.
    pub mounted: EventHandler<(usize, MountedEvent)>,
}

/// Every drawn group, the stop `current` highlighted.
pub(crate) fn draw_groups<T>(
    shown: &[ShownGroup<'_, T>],
    current: usize,
    events: StopEvents,
) -> Element {
    let drawn: Vec<Element> = shown
        .iter()
        .map(|group| draw_group(group, current, events))
        .collect();
    rsx! {
        for (key , group) in drawn.into_iter().enumerate() {
            Fragment { key: "{key}", {group} }
        }
    }
}

/// One group: its header, then its rows or its grid.
fn draw_group<T>(shown: &ShownGroup<'_, T>, current: usize, events: StopEvents) -> Element {
    let first = shown.first;
    let local = current.checked_sub(first);
    let (body, size) = match &shown.body {
        Body::Rows(lines) => {
            let size = choices_len(lines);
            let body = render_lines(
                lines,
                MenuKind::Rich.row(),
                Drawn {
                    selected: local,
                    open: None,
                    onpick: EventHandler::new(move |at: usize| events.pick.call(first + at)),
                    onpoint: EventHandler::new(move |(at, _): (usize, Point)| {
                        events.point.call(first + at)
                    }),
                    onmounted: EventHandler::new(move |(at, event): (usize, MountedEvent)| {
                        events.mounted.call((first + at, event))
                    }),
                    onrelease: None,
                },
            );
            (body, size)
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
            kind: HeaderKind::Menu,
            text: shown.group.title.clone(),
            action: shown.group.action.clone(),
            action_selection: match shown.group.action {
                Some(_) => action_selection,
                None => Selection::Unselected,
            },
        }
        {body}
    }
}
