//! What can go wrong launching, rendering or snapshotting.

/// A native render that did not happen.
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    /// The renderer could not be created (no adapter, no surface).
    #[error("renderer: {0}")]
    Renderer(String),
    /// The snapshot could not be encoded or written.
    #[error("snapshot: {0}")]
    Image(#[from] image::ImageError),
}

/// A window [`open_window`](crate::open_window) could not ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum OpenWindowError {
    /// Not inside a window `launch` runs: the harness, a snapshot, a webview, or outside any
    /// component.
    #[error("no window event loop to open a window on")]
    NoHost,
}
