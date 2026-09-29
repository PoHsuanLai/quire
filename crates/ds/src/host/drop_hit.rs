//! What the host found under a point of a file drag.

/// Which registered target the host found under a point: the innermost one whose element is
/// the element there or one of its ancestors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropHit {
    /// The target at this index of the list the host was given.
    Target(usize),
    /// No target is there.
    Nothing,
    /// The document is busy (rendering): keep what was found last.
    Busy,
}
