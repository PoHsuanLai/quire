//! How a virtual table is dressed: the grid it has always been, or Finder's list view.

use ds_core::word::Word;

/// How a `VirtualTable` looks. The rules (`ColumnRules`, `HeaderRule`) are drawn as they are
/// asked for in every look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum TableLook {
    /// The grid: the selected row fills the table's width and the header's titles are bold.
    #[default]
    Plain,
    /// Finder's list view and the SwiftUI `Table`: faint alternating stripes tell the rows
    /// apart. The selected row is a rounded block inset from the table's edges, the header is
    /// quiet (regular titles in the secondary ink, the sort indicator on the sorted column only)
    /// and the row under the pointer takes a soft tint.
    Striped,
    /// The same list without stripes (Music).
    Unstriped,
}

impl TableLook {
    /// The value the `data-look` attribute carries; `None` for the plain grid, so the table's
    /// markup is unchanged.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            TableLook::Plain => None,
            TableLook::Striped | TableLook::Unstriped => Some(self.slug()),
        }
    }

    /// Whether `row` is one of the striped rows: every other, counted from the first, so a row
    /// keeps its stripe as the window of mounted rows slides.
    pub(crate) fn striped(self, row: usize) -> bool {
        self == TableLook::Striped && row % 2 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::TableLook;

    #[test]
    fn only_the_list_looks_are_marked_and_only_striped_stripes_every_other_row() {
        // (look, attribute, row 0 striped, row 1 striped)
        let cases = [
            (TableLook::Plain, None, false, false),
            (TableLook::Striped, Some("striped"), false, true),
            (TableLook::Unstriped, Some("unstriped"), false, false),
        ];
        for (look, attribute, first, second) in cases {
            assert_eq!(look.attribute(), attribute, "{look:?}");
            assert_eq!(look.striped(0), first, "{look:?} row 0");
            assert_eq!(look.striped(1), second, "{look:?} row 1");
            assert_eq!(look.striped(10_001), second, "{look:?} row 10001");
        }
    }
}
