//! What can stop a snapshot run.

use std::path::PathBuf;

/// A snapshot run that could not finish.
#[derive(Debug, thiserror::Error)]
pub enum GalleryError {
    /// The renderer failed on a page.
    #[error("rendering {name}: {source}")]
    Render {
        /// The picture being made.
        name: String,
        /// What the renderer said.
        source: ds_native::NativeError,
    },
    /// A picture or the sheet could not be written.
    #[error("writing {path}: {source}")]
    Write {
        /// Where.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// A picture could not be encoded.
    #[error("encoding {path}: {source}")]
    Encode {
        /// Where it was going.
        path: PathBuf,
        /// Why.
        source: image::ImageError,
    },
}
