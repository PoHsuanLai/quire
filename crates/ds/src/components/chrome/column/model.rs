//! Column's alignment, as data.

use ds_core::word::Word;

/// Where a column puts its children across its width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ColumnAlign {
    /// Each child as wide as the column.
    #[default]
    Stretch,
    /// Children at the leading edge, as wide as their content.
    Start,
    /// Children centred, as wide as their content.
    Center,
    /// Children at the trailing edge, as wide as their content.
    End,
}
