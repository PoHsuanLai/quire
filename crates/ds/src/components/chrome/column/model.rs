//! Column's alignment, gap and extent, as data.

use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::spacing::SpacingToken;

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

/// The space between a column's children: none, or a step of the spacing scale (which has no
/// zero step of its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColumnGap {
    /// Children touch.
    None,
    /// This step between children.
    Step(SpacingToken),
}

impl Default for ColumnGap {
    fn default() -> Self {
        ColumnGap::Step(SpacingToken::S8)
    }
}

impl From<SpacingToken> for ColumnGap {
    fn from(step: SpacingToken) -> Self {
        ColumnGap::Step(step)
    }
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
