//! Why a desktop file was not read.

use crate::ReadCap;

/// A desktop file that was refused whole.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DesktopFileError {
    /// The text is longer than the caller's cap; nothing of it was read.
    #[error("the file is {size} bytes, over the limit of {cap}")]
    TooBig {
        /// The text's length in bytes.
        size: u64,
        /// The cap it went over.
        cap: ReadCap,
    },
}
