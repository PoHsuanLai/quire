//! A row's own part in a motion of its list's rows (the command palette's Show More and Show Less,
//! design/26-DETAILS.md section 5.6): coming in as an added row, or the last row kept, after which
//! everything heals up.

/// A row's part in a motion of the rows around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RowMotion {
    /// No part.
    #[default]
    Still,
    /// An added row coming in (`row-in`).
    In,
    /// The row everything after heals up to.
    HealFrom,
}

impl RowMotion {
    /// The `data-row-motion` word.
    pub(crate) fn attr(self) -> Option<&'static str> {
        match self {
            RowMotion::Still => None,
            RowMotion::In => Some("in"),
            RowMotion::HealFrom => Some("heal-from"),
        }
    }
}
