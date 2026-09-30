//! A row's height (design/30 section 1.6): table and list rows are 24 tall, settings rows 44; a
//! card row has the height of the content it lays out.

use ds_core::word::Word;

/// How tall a row is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum RowSize {
    /// 24: a table or list row, one line.
    #[default]
    Compact,
    /// 44: a settings row, with room for a detail line.
    Settings,
    /// A card: a row that lays its own `content` out, edge to edge (a mail thread).
    Card,
}
