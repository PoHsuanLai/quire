//! What a caller hands a `ColumnBrowser`: the columns, their rows and what a pick reports.

use crate::components::content::icon_source::IconSource;
use ds_core::selection::{Pick, RowSelection};

/// Whether a row has anything under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BrowserRowKind {
    /// Nothing under it (a file): picking it ends the columns and shows the preview.
    #[default]
    Leaf,
    /// Something under it (a folder): the row ends in a chevron and the app supplies its column.
    Branch,
}

/// One row of a column.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserRow<K> {
    /// The row's identity.
    pub key: K,
    /// The name, cut from the middle to the column's width so a file name keeps its extension. It
    /// is what type-to-select matches.
    pub name: String,
    /// The glyph or thumbnail before the name.
    pub icon: IconSource,
    /// Whether the row has a column of its own.
    pub kind: BrowserRowKind,
}

impl<K> BrowserRow<K> {
    /// A row with nothing under it.
    pub fn leaf(key: K, name: impl Into<String>, icon: IconSource) -> Self {
        BrowserRow {
            key,
            name: name.into(),
            icon,
            kind: BrowserRowKind::Leaf,
        }
    }

    /// A row with a column under it.
    pub fn branch(key: K, name: impl Into<String>, icon: IconSource) -> Self {
        BrowserRow {
            kind: BrowserRowKind::Branch,
            ..BrowserRow::leaf(key, name, icon)
        }
    }
}

/// One column: its rows and which of them are selected (row indexes, as `VirtualTable`'s are).
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserColumn<K> {
    /// The rows, top to bottom.
    pub rows: Vec<BrowserRow<K>>,
    /// The selected rows. The cursor, the row the keys rest on, is the selection's.
    pub selection: RowSelection,
}

impl<K> BrowserColumn<K> {
    /// A column with nothing selected.
    pub fn new(rows: Vec<BrowserRow<K>>) -> Self {
        BrowserColumn {
            rows,
            selection: RowSelection::none(),
        }
    }

    /// The same column with `selection`.
    pub fn selected(self, selection: RowSelection) -> Self {
        BrowserColumn { selection, ..self }
    }
}

/// A row reached in a column, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnPick {
    /// The column, counted from the first.
    pub column: usize,
    /// The row in it and how it was reached.
    pub pick: Pick,
}
