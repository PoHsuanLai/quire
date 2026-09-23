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
