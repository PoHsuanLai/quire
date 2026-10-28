//! Column's alignment, gap and extent, as data.

use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

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

/// How far a column reaches along its own height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ColumnExtent {
    /// As tall as its children.
    #[default]
    Content,
    /// As wide and as tall as its parent.
    Fill,
    /// Exactly as tall as a control of this size.
    Fixed(ControlSize),
}

impl ColumnExtent {
    /// The `data-extent` and `data-size` attributes that draw this extent.
    pub(crate) fn attributes(self) -> (Option<&'static str>, Option<&'static str>) {
        match self {
            ColumnExtent::Content => (None, None),
            ColumnExtent::Fill => (Some("fill"), None),
            ColumnExtent::Fixed(size) => (Some("fixed"), Some(size.slug())),
        }
    }
}
