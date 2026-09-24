//! The one error quire's design system returns: input it was handed that it cannot draw.

use std::path::PathBuf;

/// Why the design system refused a value at its boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DsError {
    /// An external icon's URL is neither `data:` nor `file:`: the only schemes a quire
    /// document's net provider answers (ds-native's `LocalNet`), so anything else would draw
    /// nothing, silently.
    #[error("an icon URL must be data: or file:, not {url:?}")]
    IconScheme {
        /// The URL as given.
        url: String,
    },
    /// A `file:` icon was given a relative path, which has no meaning as a URL.
    #[error("an icon file path must be absolute: {path:?}")]
    IconPathRelative {
        /// The path as given.
        path: PathBuf,
    },
    /// An external icon's bytes are not a PNG the classifier can read.
    #[error("an icon's bytes are not a readable PNG: {reason}")]
    IconDecode {
        /// What the decoder said.
        reason: String,
    },
}
