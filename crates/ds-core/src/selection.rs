//! Which rows of a list or a table are selected, and how a click or a key changes it (Finder's
//! list view, Numbers, `NSTableView`). Pure arithmetic over row indexes, so any app's table can
//! use it and a table of a million rows stays cheap: the selection is a handful of ranges, never
//! one entry per row, and "is this row selected" is a binary search.
//!
//! A selection has an anchor (where a shift-click or shift-arrow measures from) and a cursor (the
//! row the keys rest on, and the one that moved last). A [`Pick`] says which row was reached and
//! how: it replaces the selection, toggles the row in it, or extends it from the anchor.

// A selection is a list of ranges, and often holds just one: `vec![a..b]` is meant.
#![allow(clippy::single_range_in_vec_init)]

use crate::command::holds_primary;
use chordkit::Platform;
use keyboard_types::Modifiers;
use std::ops::Range;

/// How a row was reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickHow {
    /// A plain click or arrow: the row alone is selected.
    Replace,
    /// The primary modifier held (Command, or Ctrl): the row is added, or taken out if it was
    /// in.
    Toggle,
    /// Shift held: every row from the anchor to this one is selected, and the rows picked
    /// before it are not.
    Extend,
}

impl PickHow {
    /// How a press with `modifiers` held reaches a row on `platform`: Shift extends (and wins
    /// over the primary modifier), the platform's primary modifier toggles, anything else
    /// replaces.
    pub fn of_press(platform: Platform, modifiers: Modifiers) -> PickHow {
        if modifiers.contains(Modifiers::SHIFT) {
            PickHow::Extend
        } else if holds_primary(platform, modifiers) {
            PickHow::Toggle
        } else {
            PickHow::Replace
        }
    }
}

/// A row reached, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pick {
    /// The row's index.
    pub row: usize,
    /// How it was reached.
    pub how: PickHow,
}

/// The rows selected: sorted, disjoint ranges that never touch, plus the anchor and the cursor.
/// Empty by default.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowSelection {
    anchor: Option<usize>,
    cursor: Option<usize>,
    ranges: Vec<Range<usize>>,
}

impl RowSelection {
    /// Nothing selected.
    pub fn none() -> Self {
        Self::default()
    }

    /// Just `row`, which is both anchor and cursor.
    pub fn single(row: usize) -> Self {
        RowSelection {
            anchor: Some(row),
            cursor: Some(row),
            ranges: vec![row..row.saturating_add(1)],
        }
    }

    /// Exactly `rows`, in any order and with repeats, with the given anchor and cursor kept as
    /// they are (they need not be among the rows).
    pub fn of_rows(
        anchor: Option<usize>,
        cursor: Option<usize>,
        rows: impl IntoIterator<Item = usize>,
    ) -> Self {
        let mut rows: Vec<usize> = rows.into_iter().collect();
        rows.sort_unstable();
        rows.dedup();
        let mut ranges: Vec<Range<usize>> = Vec::new();
        for row in rows {
            match ranges.last_mut() {
                Some(last) if last.end == row => last.end = row + 1,
                Some(_) | None => ranges.push(row..row + 1),
            }
        }
        RowSelection {
            anchor,
            cursor,
            ranges,
        }
    }

    /// Exactly the rows in `ranges`, in any order, overlapping or touching ones joined, with the
    /// given anchor and cursor kept as they are. A selection of many rows costs its ranges, not
    /// its rows.
    pub fn of_ranges(
        anchor: Option<usize>,
        cursor: Option<usize>,
        ranges: impl IntoIterator<Item = Range<usize>>,
    ) -> Self {
        let mut ranges: Vec<Range<usize>> = ranges
            .into_iter()
            .filter(|range| range.start < range.end)
            .collect();
        ranges.sort_by_key(|range| range.start);
        let mut joined: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
        for range in ranges {
            match joined.last_mut() {
                Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
                Some(_) | None => joined.push(range),
            }
        }
        RowSelection {
            anchor,
            cursor,
            ranges: joined,
        }
    }

    /// This selection with the rows in `more` added. The anchor stays (the first added row when
    /// there was none) and the cursor becomes the last added row's, as a drag across rows leaves
    /// them.
    #[must_use]
    pub fn united(self, more: &[Range<usize>]) -> Self {
        let added: Vec<Range<usize>> = more
            .iter()
            .filter(|range| range.start < range.end)
            .cloned()
            .collect();
        let first = added.iter().map(|range| range.start).min();
        let last = added.iter().map(|range| range.end - 1).max();
        RowSelection::of_ranges(
            self.anchor.or(first),
            last.or(self.cursor),
            self.ranges.into_iter().chain(added),
        )
    }

    /// Every one of `len` rows. The anchor and the cursor stay where they were, else they are
    /// the first row; no rows select nothing.
    #[must_use]
    pub fn all(self, len: usize) -> Self {
        if len == 0 {
            return RowSelection::none();
        }
        RowSelection {
            anchor: self.anchor.filter(|at| *at < len).or(Some(0)),
            cursor: self.cursor.filter(|at| *at < len).or(Some(0)),
            ranges: vec![0..len],
        }
    }

    /// The selection after `pick`.
    #[must_use]
    pub fn picked(self, pick: Pick) -> Self {
        let Pick { row, how } = pick;
        match (how, self.anchor) {
            (PickHow::Replace, _) | (PickHow::Extend, None) => RowSelection::single(row),
            (PickHow::Toggle, _) => self.toggled(row),
            (PickHow::Extend, Some(anchor)) => RowSelection {
                anchor: Some(anchor),
                cursor: Some(row),
                ranges: vec![anchor.min(row)..anchor.max(row).saturating_add(1)],
            },
        }
    }

    /// Only the cursor's row, or nothing when there is no cursor: what Esc does to a selection
    /// of several rows.
    #[must_use]
    pub fn narrowed(self) -> Self {
        self.cursor
            .map_or_else(RowSelection::none, RowSelection::single)
    }

    /// Whether `row` is selected.
    pub fn contains(&self, row: usize) -> bool {
        let at = self.ranges.partition_point(|range| range.end <= row);
        self.ranges.get(at).is_some_and(|range| range.start <= row)
    }

    /// How many rows are selected.
    pub fn count(&self) -> usize {
        self.ranges
            .iter()
            .map(|range| range.end - range.start)
            .sum()
    }

    /// Whether no row is selected.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// The selected rows in ascending order, one at a time.
    pub fn rows(&self) -> impl Iterator<Item = usize> + '_ {
        self.ranges.iter().flat_map(Clone::clone)
    }

    /// The selected ranges, ascending, disjoint, and never touching.
    pub fn ranges(&self) -> &[Range<usize>] {
        &self.ranges
    }

    /// The row a shift-click or shift-arrow measures from.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    /// The row the keys rest on.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// `row` added if it was out, taken out if it was in; it becomes the anchor and the cursor.
    fn toggled(mut self, row: usize) -> Self {
        let at = self.ranges.partition_point(|range| range.end <= row);
        let inside = self
            .ranges
            .get(at)
            .filter(|range| range.start <= row)
            .cloned();
        match inside {
            Some(range) => {
                self.ranges.remove(at);
                let after = row + 1..range.end;
                if !after.is_empty() {
                    self.ranges.insert(at, after);
                }
                let before = range.start..row;
                if !before.is_empty() {
                    self.ranges.insert(at, before);
                }
            }
            None => {
                let joins_before = at > 0 && self.ranges[at - 1].end == row;
                let joins_after = self
                    .ranges
                    .get(at)
                    .is_some_and(|next| next.start == row + 1);
                match (joins_before, joins_after) {
                    (true, true) => {
                        let end = self.ranges.remove(at).end;
                        self.ranges[at - 1].end = end;
                    }
                    (true, false) => self.ranges[at - 1].end = row + 1,
                    (false, true) => self.ranges[at].start = row,
                    (false, false) => self.ranges.insert(at, row..row + 1),
                }
            }
        }
        self.anchor = Some(row);
        self.cursor = Some(row);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Pick, PickHow, RowSelection};
    use chordkit::Platform;
    use keyboard_types::Modifiers;

    fn pick(row: usize, how: PickHow) -> Pick {
        Pick { row, how }
    }

    fn ranges(selection: &RowSelection) -> Vec<std::ops::Range<usize>> {
        selection.ranges().to_vec()
    }

    #[test]
    fn a_pick_replaces_toggles_or_extends_from_the_anchor() {
        use PickHow::{Extend, Replace, Toggle};
        // (name, picks in order, wanted ranges, anchor, cursor)
        type Case = (
            &'static str,
            Vec<Pick>,
            Vec<std::ops::Range<usize>>,
            Option<usize>,
            Option<usize>,
        );
        let cases: Vec<Case> = vec![
            (
                "a click selects the row",
                vec![pick(4, Replace)],
                vec![4..5],
                Some(4),
                Some(4),
            ),
            (
                "another click moves the selection",
                vec![pick(4, Replace), pick(9, Replace)],
                vec![9..10],
                Some(9),
                Some(9),
            ),
            (
                "toggling adds a row apart",
                vec![pick(2, Replace), pick(7, Toggle)],
                vec![2..3, 7..8],
                Some(7),
                Some(7),
            ),
            (
                "toggling next to a range joins it",
                vec![pick(2, Replace), pick(3, Toggle)],
                vec![2..4],
                Some(3),
                Some(3),
            ),
            (
                "toggling between two ranges joins both",
                vec![pick(2, Replace), pick(4, Toggle), pick(3, Toggle)],
                vec![2..5],
                Some(3),
                Some(3),
            ),
            (
                "toggling a selected row takes it out",
                vec![pick(2, Replace), pick(5, Toggle), pick(5, Toggle)],
                vec![2..3],
                Some(5),
                Some(5),
            ),
            (
                "toggling the middle of a range splits it",
                vec![pick(2, Replace), pick(6, Extend), pick(4, Toggle)],
                vec![2..4, 5..7],
                Some(4),
                Some(4),
            ),
            (
                "toggling the start of a range shortens it",
                vec![pick(2, Replace), pick(6, Extend), pick(2, Toggle)],
                vec![3..7],
                Some(2),
                Some(2),
            ),
            (
                "toggling the end of a range shortens it",
                vec![pick(2, Replace), pick(6, Extend), pick(6, Toggle)],
                vec![2..6],
                Some(6),
                Some(6),
            ),
            (
                "toggling the only row leaves nothing",
                vec![pick(2, Replace), pick(2, Toggle)],
                vec![],
                Some(2),
                Some(2),
            ),
            (
                "extending selects from the anchor down",
                vec![pick(2, Replace), pick(5, Extend)],
                vec![2..6],
                Some(2),
                Some(5),
            ),
            (
                "extending selects from the anchor up",
                vec![pick(5, Replace), pick(2, Extend)],
                vec![2..6],
                Some(5),
                Some(2),
            ),
            (
                "extending again measures from the same anchor",
                vec![pick(2, Replace), pick(8, Extend), pick(4, Extend)],
                vec![2..5],
                Some(2),
                Some(4),
            ),
            (
                "extending past the anchor flips the range",
                vec![pick(5, Replace), pick(8, Extend), pick(3, Extend)],
                vec![3..6],
                Some(5),
                Some(3),
            ),
            (
                "extending drops the rows picked apart",
                vec![pick(0, Replace), pick(9, Toggle), pick(3, Extend)],
                vec![3..10],
                Some(9),
                Some(3),
            ),
            (
                "extending with nothing selected is a click",
                vec![pick(6, Extend)],
                vec![6..7],
                Some(6),
                Some(6),
            ),
            (
                "a toggle after an extend keeps the range and moves the anchor",
                vec![
                    pick(1, Replace),
                    pick(3, Extend),
                    pick(8, Toggle),
                    pick(9, Extend),
                ],
                vec![8..10],
                Some(8),
                Some(9),
            ),
        ];
        for (name, picks, want, anchor, cursor) in cases {
            let got = picks
                .into_iter()
                .fold(RowSelection::none(), RowSelection::picked);
            assert_eq!(ranges(&got), want, "{name}: ranges");
            assert_eq!(got.anchor(), anchor, "{name}: anchor");
            assert_eq!(got.cursor(), cursor, "{name}: cursor");
        }
    }

    #[test]
    fn a_selection_answers_what_it_holds_without_listing_rows() {
        let held = RowSelection::of_rows(None, None, [9, 2, 3, 4, 3, 7]);
        assert_eq!(ranges(&held), vec![2..5, 7..8, 9..10]);
        // (row, selected)
        let cases = [
            (0, false),
            (1, false),
            (2, true),
            (4, true),
            (5, false),
            (6, false),
            (7, true),
            (8, false),
            (9, true),
            (10, false),
            (usize::MAX, false),
        ];
        for (row, want) in cases {
            assert_eq!(held.contains(row), want, "row {row}");
        }
        assert_eq!(held.count(), 5);
        assert_eq!(held.rows().collect::<Vec<_>>(), vec![2, 3, 4, 7, 9]);
        assert!(!held.is_empty());
        assert!(RowSelection::none().is_empty());
        assert!(!RowSelection::none().contains(0));
    }

    #[test]
    fn a_million_rows_are_one_range() {
        let all = RowSelection::none().all(1_000_000);
        assert_eq!(ranges(&all), vec![0..1_000_000]);
        assert_eq!(all.count(), 1_000_000);
        assert!(all.contains(999_999));
        assert!(!all.contains(1_000_000));
    }

    #[test]
    fn select_all_keeps_the_anchor_and_cursor_and_esc_narrows_to_the_cursor() {
        // (name, before, len, wanted ranges, anchor, cursor)
        let from = |anchor, cursor, rows: &[usize]| {
            RowSelection::of_rows(anchor, cursor, rows.iter().copied())
        };
        let cases = [
            (
                "none",
                from(None, None, &[]),
                4,
                vec![0..4],
                Some(0),
                Some(0),
            ),
            (
                "keeps the cursor",
                from(Some(1), Some(2), &[1, 2]),
                4,
                vec![0..4],
                Some(1),
                Some(2),
            ),
            (
                "a cursor past the end goes",
                from(Some(8), Some(9), &[8, 9]),
                4,
                vec![0..4],
                Some(0),
                Some(0),
            ),
            (
                "no rows",
                from(Some(1), Some(1), &[1]),
                0,
                vec![],
                None,
                None,
            ),
        ];
        for (name, before, len, want, anchor, cursor) in cases {
            let got = before.all(len);
            assert_eq!(ranges(&got), want, "{name}");
            assert_eq!((got.anchor(), got.cursor()), (anchor, cursor), "{name}");
        }
        let several = RowSelection::none()
            .picked(pick(1, PickHow::Replace))
            .picked(pick(4, PickHow::Extend));
        let narrow = several.narrowed();
        assert_eq!(ranges(&narrow), vec![4..5]);
        assert_eq!(narrow.anchor(), Some(4));
        assert!(RowSelection::none().narrowed().is_empty());
    }

    #[test]
    fn shift_extends_the_primary_modifier_toggles_and_a_bare_press_replaces() {
        // (modifiers held, wanted)
        let cases = [
            (Modifiers::empty(), PickHow::Replace),
            (Modifiers::SHIFT, PickHow::Extend),
            (Modifiers::SUPER, PickHow::Toggle),
            (Modifiers::SUPER | Modifiers::SHIFT, PickHow::Extend),
            (Modifiers::ALT, PickHow::Replace),
        ];
        for (held, want) in cases {
            assert_eq!(PickHow::of_press(Platform::MacOs, held), want, "{held:?}");
        }
    }

    #[test]
    fn ranges_are_joined_and_a_union_keeps_the_anchor_and_moves_the_cursor_to_the_last_added() {
        let joined = RowSelection::of_ranges(None, None, [5..8, 1..3, 3..4, 7..10, 12..12]);
        assert_eq!(
            ranges(&joined),
            vec![1..4, 5..10],
            "touching and overlapping join"
        );
        let base = RowSelection::of_rows(Some(2), Some(2), [2]);
        let added = base.united(&[6..8, 4..5]);
        assert_eq!(ranges(&added), vec![2..3, 4..5, 6..8]);
        assert_eq!((added.anchor(), added.cursor()), (Some(2), Some(7)));
        let from_nothing = RowSelection::none().united(&[3..5]);
        assert_eq!(
            (from_nothing.anchor(), from_nothing.cursor()),
            (Some(3), Some(4))
        );
        let nothing_added = RowSelection::single(1).united(&[]);
        assert_eq!(ranges(&nothing_added), vec![1..2]);
    }
}
