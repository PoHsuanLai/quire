//! How one attempt to move the keyboard focus through the host went.

/// One attempt at moving the focus through the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Focused {
    /// The element has the focus.
    Done,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// The host cannot focus this element (not its node, or gone).
    Unknown,
}
