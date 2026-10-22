//! What dragging rows of a `VirtualTable` decides, pure: which rows travel, where a pointer at a
//! height in the content would drop them, and what the modifiers make of the drop.
//!
//! Every row has one height, so a drop target is found by arithmetic on the pointer's height and
//! the rows' pitch: no row has to be mounted, which is the point in a table that mounts only the
//! rows in view.

use crate::file_drop::drag::DropOperation;
use chordkit::Platform;
use dioxus::prelude::Modifiers;
use ds_core::selection::RowSelection;

/// What a row is as a place to drop on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowDropKind {
    /// A row that holds others (a folder): dropping on it drops into it, and it is highlighted.
    Container,
    /// A plain row: nothing is dropped onto it.
    #[default]
    Item,
}

/// Whether the gaps between rows are places to drop (a list the person can reorder), or only the
/// container rows are (files moved into folders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowGaps {
    /// A line is drawn in the nearest gap and the drop lands there.
    Offered,
    /// Only a container row takes a drop.
    #[default]
    Withheld,
}

/// Where a drag of rows would land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowDropTarget {
    /// Into the container row (its index in the table).
    Onto(usize),
    /// Into the gap before row `n` (`n == rows` is the gap after the last), drawn as a line.
    Between(usize),
}

/// What the owner hears when rows are let go on a target.
#[derive(Debug, Clone, PartialEq)]
pub struct RowDrop {
    /// The rows that were dragged, in order: the whole selection when the drag began on a selected
    /// row, else that row alone.
    pub rows: Vec<usize>,
    /// Where they were let go.
    pub target: RowDropTarget,
    /// What the modifiers held at the release ask for: Option copies, Command moves, both link,
    /// and none moves.
    pub operation: DropOperation,
}

/// The rows a drag that began on `pressed` carries: the selection when it holds `pressed`
/// (dragging a selected row drags them all), else `pressed` alone.
pub(crate) fn dragged_rows(pressed: usize, selection: Option<&RowSelection>) -> Vec<usize> {
    match selection {
        Some(chosen) if chosen.contains(pressed) => chosen.rows().collect(),
        Some(_) | None => vec![pressed],
    }
}

/// The part of a container row (from its top, as a fraction of its height) that is the row
/// itself when gaps are offered: the middle half. The quarters above and below belong to the gaps.
const ONTO: std::ops::Range<f32> = 0.25..0.75;

/// Where a drop at `y` (from the content's top) lands among `len` rows `pitch` tall, if anywhere.
/// `kind` says what each row is.
pub(crate) fn drop_target(
    y: f32,
    pitch: f32,
    len: usize,
    gaps: RowGaps,
    kind: impl Fn(usize) -> RowDropKind,
) -> Option<RowDropTarget> {
    if !(y.is_finite() && pitch > 0.0) || len == 0 || y < 0.0 {
        return None;
    }
    let at = y / pitch;
    let row = at.floor() as usize;
    if row >= len {
        return (gaps == RowGaps::Offered).then_some(RowDropTarget::Between(len));
    }
    let into = at - row as f32;
    let nearest = RowDropTarget::Between(row + usize::from(into >= 0.5));
    match (kind(row), gaps) {
        (RowDropKind::Container, RowGaps::Withheld) => Some(RowDropTarget::Onto(row)),
        (RowDropKind::Container, RowGaps::Offered) => Some(match ONTO.contains(&into) {
            true => RowDropTarget::Onto(row),
            false => nearest,
        }),
        (RowDropKind::Item, RowGaps::Offered) => Some(nearest),
        (RowDropKind::Item, RowGaps::Withheld) => None,
    }
}

/// `target` unless it would drop the rows onto themselves (a folder into itself) or into a gap
/// inside or beside their own block (no change).
pub(crate) fn allowed(target: RowDropTarget, rows: &[usize]) -> Option<RowDropTarget> {
    let moves = match target {
        RowDropTarget::Onto(row) => !rows.contains(&row),
        RowDropTarget::Between(gap) => {
            let after = gap > 0 && rows.contains(&(gap - 1));
            let before = rows.contains(&gap);
            !(after || before)
        }
    };
    moves.then_some(target)
}

/// What a drop made with `modifiers` held does on `platform`: Option copies, Command moves, both
/// make a link, and no modifier moves.
pub(crate) fn operation_of(platform: Platform, modifiers: Modifiers) -> DropOperation {
    DropOperation::forced_by(platform, modifiers).unwrap_or(DropOperation::Move)
}

#[cfg(test)]
mod tests {
    use super::{
        RowDropKind, RowDropTarget, RowGaps, allowed, dragged_rows, drop_target, operation_of,
    };
    use crate::file_drop::drag::DropOperation;
    use chordkit::Platform;
    use dioxus::prelude::Modifiers;
    use ds_core::selection::RowSelection;

    /// Rows 1 and 3 are folders.
    fn kind(row: usize) -> RowDropKind {
        match row {
            1 | 3 => RowDropKind::Container,
            _ => RowDropKind::Item,
        }
    }

    #[test]
    fn a_selected_row_drags_the_whole_selection_and_any_other_drags_itself() {
        let chosen = RowSelection::of_rows(Some(2), Some(2), [2, 3, 7]);
        assert_eq!(dragged_rows(3, Some(&chosen)), vec![2, 3, 7]);
        assert_eq!(dragged_rows(5, Some(&chosen)), vec![5]);
        assert_eq!(dragged_rows(5, None), vec![5]);
    }

    #[test]
    fn a_pointer_height_finds_a_folder_or_a_gap_by_arithmetic_on_the_pitch() {
        use RowDropTarget::{Between, Onto};
        let (offered, withheld) = (RowGaps::Offered, RowGaps::Withheld);
        // (y, gaps, wanted) with rows 20 tall and 5 of them.
        let cases = [
            (-3.0, withheld, None),
            (30.0, withheld, Some(Onto(1))),
            (21.0, withheld, Some(Onto(1))),
            (50.0, withheld, None),
            (110.0, withheld, None),
            (30.0, offered, Some(Onto(1))),
            (21.0, offered, Some(Between(1))),
            (39.0, offered, Some(Between(2))),
            (49.0, offered, Some(Between(2))),
            (58.0, offered, Some(Between(3))),
            (99.0, offered, Some(Between(5))),
            (130.0, offered, Some(Between(5))),
        ];
        for (y, gaps, want) in cases {
            assert_eq!(drop_target(y, 20.0, 5, gaps, kind), want, "{y} {gaps:?}");
        }
        assert_eq!(drop_target(5.0, 0.0, 5, offered, kind), None, "no pitch");
        assert_eq!(drop_target(5.0, 20.0, 0, offered, kind), None, "no rows");
    }

    #[test]
    fn rows_are_never_dropped_on_themselves_or_into_their_own_block() {
        use RowDropTarget::{Between, Onto};
        let rows = [2, 3];
        let cases = [
            (Onto(1), Some(Onto(1))),
            (Onto(3), None),
            (Between(1), Some(Between(1))),
            (Between(2), None),
            (Between(3), None),
            (Between(4), None),
            (Between(5), Some(Between(5))),
        ];
        for (target, want) in cases {
            assert_eq!(allowed(target, &rows), want, "{target:?}");
        }
    }

    #[test]
    fn option_copies_command_moves_both_link_and_none_moves() {
        let mac = Platform::MacOs;
        let cases = [
            (Modifiers::empty(), DropOperation::Move),
            (Modifiers::ALT, DropOperation::Copy),
            (Modifiers::SUPER, DropOperation::Move),
            (Modifiers::ALT | Modifiers::SUPER, DropOperation::Link),
            (Modifiers::SHIFT, DropOperation::Move),
        ];
        for (modifiers, want) in cases {
            assert_eq!(operation_of(mac, modifiers), want, "{modifiers:?}");
        }
    }
}
