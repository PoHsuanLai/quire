//! Where the command palette's cursor can rest, and how the keys move it (sill Q291, Q294):
//! pure, beside `command_palette`.
//!
//! A *stop* is a row (an item or a submenu parent, as a menu's choice), an emoji cell, or a
//! group's header action ("Show More"). Stops are numbered in drawing order, each group's
//! action after its last row or cell: the palette's `selected`, `on_select` and
//! `on_select_rect` name a stop by that number, which for a palette of plain rows is the
//! choice number it always was. Up and Down walk the stops clamped, skipping disabled rows; in
//! a grid they move a row in two dimensions and leave it past its top and bottom rows; Left and
//! Right move only inside a grid (elsewhere they are the field's).

use crate::components::emoji_grid::EmojiCells;
use crate::components::emoji_grid_nav::{GridEdge, GridMove, GridStep, grid_step};
use crate::components::menu_lines::{
    Act, Choice, Line, Nav, Step, choices, choices_len, moved_live,
};
use crate::components::palette_group::{GroupEntries, PaletteGroup};
use crate::components::palette_lines::marked;
use crate::components::vocab::Availability;
use dioxus::prelude::EventHandler;

/// A group as drawn: its rows marked for the query, or its grid, and its first stop.
pub(crate) struct ShownGroup<'a, T: 'static> {
    /// The group as given.
    pub group: &'a PaletteGroup<T>,
    /// What it draws.
    pub body: Body<'a, T>,
    /// The number of its first stop.
    pub first: usize,
}

/// A drawn group's body.
pub(crate) enum Body<'a, T> {
    /// Its rows, marked where the query matches.
    Rows(Vec<Line<'a, T>>),
    /// Its grid.
    Grid(&'a EmojiCells<T>),
}

/// The groups that draw (the empty ones skipped), each with its first stop.
pub(crate) fn shown_groups<'a, T: Clone>(
    groups: &'a [PaletteGroup<T>],
    query: &str,
) -> Vec<ShownGroup<'a, T>> {
    let mut first = 0;
    groups
        .iter()
        .filter(|group| !group.entries.is_empty())
        .map(|group| {
            let body = match &group.entries {
                GroupEntries::List(entries) => Body::Rows(marked(entries, query)),
                GroupEntries::Grid(grid) => Body::Grid(grid),
            };
            let shown = ShownGroup { group, body, first };
            first += stop_count(&shown);
            shown
        })
        .collect()
}

/// How many stops a drawn group holds: its choices or cells, and its action.
fn stop_count<T: Clone>(shown: &ShownGroup<'_, T>) -> usize {
    let body = match &shown.body {
        Body::Rows(lines) => choices_len(lines),
        Body::Grid(grid) => grid.cells.len(),
    };
    body + usize::from(shown.group.action.is_some())
}

/// Where the cursor can rest.
#[derive(Clone, PartialEq)]
pub(crate) enum Stop<T> {
    /// A row: what picking it does, and whether it can.
    Row(Choice<T>),
    /// An emoji cell, yielding this value.
    Cell(T),
    /// A group's header action.
    Action(EventHandler<()>),
}

impl<T> Stop<T> {
    /// Whether the cursor may rest here: a disabled row is skipped.
    pub(crate) fn availability(&self) -> Availability {
        match self {
            Stop::Row(choice) => choice.availability,
            Stop::Cell(_) | Stop::Action(_) => Availability::Enabled,
        }
    }
}

/// Every stop of the drawn groups, in order.
pub(crate) fn stops<T: Clone>(shown: &[ShownGroup<'_, T>]) -> Vec<Stop<T>> {
    shown
        .iter()
        .flat_map(|group| {
            let body: Vec<Stop<T>> = match &group.body {
                Body::Rows(lines) => choices(lines).into_iter().map(Stop::Row).collect(),
                Body::Grid(grid) => grid
                    .cells
                    .iter()
                    .map(|cell| Stop::Cell(cell.value.clone()))
                    .collect(),
            };
            let action = group
                .group
                .action
                .as_ref()
                .map(|(_, run)| Stop::Action(*run));
            body.into_iter().chain(action)
        })
        .collect()
}

/// A grid among the stops: its first cell's stop, its cell count and its width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GridSpan {
    pub first: usize,
    pub count: usize,
    pub columns: u8,
}

/// The grids of the drawn groups.
pub(crate) fn grid_spans<T>(shown: &[ShownGroup<'_, T>]) -> Vec<GridSpan> {
    shown
        .iter()
        .filter_map(|group| match &group.body {
            Body::Grid(grid) => Some(GridSpan {
                first: group.first,
                count: grid.cells.len(),
                columns: grid.columns,
            }),
            Body::Rows(_) => None,
        })
        .collect()
}

/// An arrow key for the palette's cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Travel {
    /// Up or Down.
    Vertical(Step),
    /// Left (`Step::Up`) or Right (`Step::Down`).
    Side(Step),
}

/// Where `travel` takes the cursor from `current`, among stops whose availability is `live`;
/// `None` when the palette leaves the key alone (Left and Right outside a grid).
pub(crate) fn travel(
    current: usize,
    travel: Travel,
    live: &[Availability],
    grids: &[GridSpan],
) -> Option<usize> {
    let grid = grids
        .iter()
        .find(|span| (span.first..span.first + span.count).contains(&current));
    match (grid, travel) {
        (None, Travel::Side(_)) => None,
        (None, Travel::Vertical(step)) => Some(moved_live(Nav::Clamp, current, live, step)),
        (Some(span), travel) => Some(in_grid(*span, current, travel, live)),
    }
}

/// A move from a cell of `span`: inside the grid, or out past its top or bottom row to the
/// nearest live stop that way (staying on the cell when there is none).
fn in_grid(span: GridSpan, current: usize, travel: Travel, live: &[Availability]) -> usize {
    let step = match travel {
        Travel::Vertical(Step::Up) => GridStep::Up,
        Travel::Vertical(Step::Down) => GridStep::Down,
        Travel::Side(Step::Up) => GridStep::Left,
        Travel::Side(Step::Down) => GridStep::Right,
    };
    match grid_step(current - span.first, step, span.count, span.columns) {
        GridMove::To(cell) => span.first + cell,
        GridMove::Out(edge) => {
            let (from, step) = match edge {
                GridEdge::Top => (span.first, Step::Up),
                GridEdge::Bottom => (span.first + span.count - 1, Step::Down),
            };
            match moved_live(Nav::Clamp, from, live, step) {
                next if next == from => current,
                next => next,
            }
        }
    }
}

/// What Enter does on a stop.
pub(crate) enum Run<T> {
    /// Close, then pick this value.
    Pick(T),
    /// Run a header action; the palette stays.
    Action(EventHandler<()>),
    /// Nothing: a disabled row, a submenu parent, or no stop.
    Nothing,
}

/// What Enter (or a click) does on `stop`.
pub(crate) fn run_of<T: Clone>(stop: Option<&Stop<T>>) -> Run<T> {
    match stop {
        Some(Stop::Row(Choice {
            act: Act::Pick(value),
            availability: Availability::Enabled,
        })) => Run::Pick(value.clone()),
        Some(Stop::Cell(value)) => Run::Pick(value.clone()),
        Some(Stop::Action(run)) => Run::Action(*run),
        Some(Stop::Row(_)) | None => Run::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::{GridSpan, Travel, travel};
    use crate::components::menu_lines::Step::{Down, Up};
    use crate::components::vocab::Availability::{Disabled as D, Enabled as E};

    #[test]
    fn the_cursor_walks_rows_enters_a_grid_and_leaves_it() {
        // Stops: rows 0 and 1, a grid of 5 cells 3 wide at 2..=6 (rows 2..=4 and 5..=6), then
        // a disabled row 7, a row 8 and an action 9.
        let live = [E, E, E, E, E, E, E, D, E, E];
        let grids = [GridSpan {
            first: 2,
            count: 5,
            columns: 3,
        }];
        #[rustfmt::skip]
        let cases: &[(usize, Travel, Option<usize>)] = &[
            (0, Travel::Vertical(Down), Some(1)),
            (1, Travel::Vertical(Down), Some(2)),
            (0, Travel::Side(Down), None),
            (2, Travel::Side(Down), Some(3)),
            (4, Travel::Side(Down), Some(5)),
            (5, Travel::Side(Up), Some(4)),
            (3, Travel::Vertical(Down), Some(6)),
            (4, Travel::Vertical(Down), Some(6)),
            (2, Travel::Vertical(Up), Some(1)),
            (6, Travel::Vertical(Down), Some(8)),
            (5, Travel::Vertical(Down), Some(8)),
            (5, Travel::Vertical(Up), Some(2)),
            (8, Travel::Vertical(Up), Some(6)),
            (8, Travel::Vertical(Down), Some(9)),
            (9, Travel::Vertical(Down), Some(9)),
        ];
        for &(from, how, want) in cases {
            assert_eq!(travel(from, how, &live, &grids), want, "{from} {how:?}");
        }
    }

    #[test]
    fn a_grid_with_nothing_beyond_keeps_the_cursor() {
        let grids = [GridSpan {
            first: 0,
            count: 4,
            columns: 2,
        }];
        let live = [E; 4];
        assert_eq!(travel(1, Travel::Vertical(Up), &live, &grids), Some(1));
        assert_eq!(travel(3, Travel::Vertical(Down), &live, &grids), Some(3));
    }
}
