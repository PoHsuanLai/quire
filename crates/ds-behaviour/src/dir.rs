//! The direction a chord or a key steps a selection in.

/// Which way through an ordered list a step goes; both directions wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    /// Toward the end of the list (Command-Tab, Tab, Right).
    Next,
    /// Toward the start of the list (Shift-Command-Tab, Shift-Tab, Grave, Left).
    Previous,
}
