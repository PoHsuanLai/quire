//! What can go wrong painting a harness's document.

/// A paint that did not happen.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// The renderer could not be created (no adapter, no surface).
    #[error("renderer: {0}")]
    Renderer(String),
    /// The picture could not be encoded or written.
    #[error("snapshot: {0}")]
    Image(#[from] image::ImageError),
}
