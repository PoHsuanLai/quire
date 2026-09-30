//! What can go wrong opening a window.

/// A window [`open_window`](crate::open_window) could not ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum OpenWindowError {
    /// Not inside a window `launch` runs: the harness, a snapshot, or outside any
    /// component.
    #[error("no window event loop to open a window on")]
    NoHost,
}
