//! What a `Table` is given: its columns, its rows and its sort (data, and the one rule of what a
//! press on a header asks for).

use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// Which way a column sorts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum SortDirection {
    /// Smallest first: the indicator points up.
    Ascending,
    /// Largest first: the indicator points down.
    Descending,
}

impl SortDirection {
    /// The other way.
    pub fn flipped(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// The column a table is sorted by, and which way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort<C> {
    /// The column.
    pub column: C,
    /// The direction.
    pub direction: SortDirection,
}

impl<C: Clone + PartialEq> Sort<C> {
    /// What a press on `pressed`'s header asks for: the sorted column flips, any other column
    /// sorts ascending.
    pub fn after_press(current: Option<&Sort<C>>, pressed: &C) -> Sort<C> {
        match current {
            Some(sort) if &sort.column == pressed => Sort {
                column: pressed.clone(),
                direction: sort.direction.flipped(),
            },
            Some(_) | None => Sort {
                column: pressed.clone(),
                direction: SortDirection::Ascending,
            },
        }
    }
}

/// Whether a press on a column's header sorts by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Sorting {
    /// It sorts.
    #[default]
    Sortable,
    /// A plain title.
    Fixed,
}

/// Where a column's cells and title sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CellAlign {
    /// At the start: text.
    #[default]
    Leading,
    /// At the end: numbers.
    Trailing,
}

/// One column.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn<C> {
    /// Its identity, reported by a sort.
    pub id: C,
    /// Its title.
    pub title: String,
    /// Its width before anyone resizes it.
    pub width: Px,
    /// The least a resize leaves it.
    pub min_width: Px,
    /// Where its cells sit.
    pub align: CellAlign,
    /// Whether its header sorts.
    pub sorting: Sorting,
}

impl<C> TableColumn<C> {
    /// A sortable, leading-aligned column `width` wide, which a resize leaves at least 48 px.
    pub fn new(id: C, title: impl Into<String>, width: Px) -> Self {
        TableColumn {
            id,
            title: title.into(),
            width,
            min_width: Px(48.0),
            align: CellAlign::Leading,
            sorting: Sorting::Sortable,
        }
    }

    /// The same column with cells at `align`.
    pub fn aligned(self, align: CellAlign) -> Self {
        TableColumn { align, ..self }
    }

    /// The same column with `sorting`.
    pub fn sorting(self, sorting: Sorting) -> Self {
        TableColumn { sorting, ..self }
    }
}

/// One row: its identity, what a screen reader and type-to-select call it, and a cell for each
/// column.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow<K> {
    /// The row's identity, stable across renders.
    pub key: K,
    /// What type-to-select matches.
    pub label: String,
    /// Whether the keys may rest on it.
    pub availability: Availability,
    /// The cells, in the columns' order.
    pub cells: Vec<Element>,
}

impl<K> TableRow<K> {
    /// An enabled row.
    pub fn new(key: K, label: impl Into<String>, cells: Vec<Element>) -> Self {
        TableRow {
            key,
            label: label.into(),
            availability: Availability::Enabled,
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Sort, SortDirection};

    #[test]
    fn a_header_press_flips_the_sorted_column_and_sorts_another_ascending() {
        /// name, the sorted column now, the column pressed, the sort asked for.
        type Case = (
            &'static str,
            Option<(u8, SortDirection)>,
            u8,
            (u8, SortDirection),
        );
        const CASES: &[Case] = &[
            ("nothing sorted", None, 1, (1, SortDirection::Ascending)),
            (
                "the sorted column, ascending",
                Some((1, SortDirection::Ascending)),
                1,
                (1, SortDirection::Descending),
            ),
            (
                "the sorted column, descending",
                Some((1, SortDirection::Descending)),
                1,
                (1, SortDirection::Ascending),
            ),
            (
                "another column",
                Some((1, SortDirection::Descending)),
                2,
                (2, SortDirection::Ascending),
            ),
        ];
        for &(name, current, pressed, (column, direction)) in CASES {
            let current = current.map(|(column, direction)| Sort { column, direction });
            assert_eq!(
                Sort::after_press(current.as_ref(), &pressed),
                Sort { column, direction },
                "{name}"
            );
        }
    }
}
