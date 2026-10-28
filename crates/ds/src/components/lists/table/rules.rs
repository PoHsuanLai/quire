//! The lines a table draws besides its rows: between its columns, and under its header.

use ds_core::geometry::units::Px;
use ds_core::word::Word;

/// Whether a table draws a line between its columns, in the header and in every row (Numbers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ColumnRules {
    /// No lines: the columns are told apart by their alignment and gap.
    #[default]
    None,
    /// A hairline between each column and the next.
    Hairline,
}

impl ColumnRules {
    /// The value the `data-rules` attribute carries; `None` for no lines, so the table's markup
    /// is unchanged.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            ColumnRules::None => None,
            ColumnRules::Hairline => Some(self.slug()),
        }
    }
}

/// Whether a table's header has a line under it once the rows have scrolled beneath it (Finder's
/// list view).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum HeaderRule {
    /// The line shows while the rows are scrolled and not at the top.
    #[default]
    WhenScrolled,
    /// No line.
    Never,
}

/// The value of `data-scrolled`: `yes` when the content is `offset` past the top, so the rows sit
/// under the header, and `no` when the rows start at the top.
pub(crate) fn scrolled_word(offset: Px) -> &'static str {
    if offset.0 > 0.0 { "yes" } else { "no" }
}

impl HeaderRule {
    /// The value the `data-scrolled` attribute carries; `None` when the header never has a line.
    pub(crate) fn attribute(self, offset: Px) -> Option<&'static str> {
        match self {
            HeaderRule::WhenScrolled => Some(scrolled_word(offset)),
            HeaderRule::Never => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnRules, HeaderRule};
    use ds_core::geometry::units::Px;

    #[test]
    fn the_attributes_follow_the_rules_and_the_offset() {
        assert_eq!(ColumnRules::None.attribute(), None);
        assert_eq!(ColumnRules::Hairline.attribute(), Some("hairline"));
        assert_eq!(HeaderRule::Never.attribute(Px(40.0)), None);
        assert_eq!(HeaderRule::WhenScrolled.attribute(Px(0.0)), Some("no"));
        assert_eq!(HeaderRule::WhenScrolled.attribute(Px(1.0)), Some("yes"));
    }
}
