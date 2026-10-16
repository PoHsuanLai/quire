//! How much of a file a read may take.

use std::fmt;

/// The most bytes a read takes from one file. The caller chooses it for the file's kind: a
/// settings file has no business being megabytes, a document can be large. A file over the cap is
/// refused whole ([`crate::ReadError::TooBig`]), never cut short.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadCap(u64);

impl ReadCap {
    /// A cap of `count` bytes.
    pub const fn bytes(count: u64) -> Self {
        ReadCap(count)
    }

    /// A cap of `count` KiB.
    pub const fn kib(count: u64) -> Self {
        ReadCap(count.saturating_mul(1024))
    }

    /// A cap of `count` MiB.
    pub const fn mib(count: u64) -> Self {
        ReadCap(count.saturating_mul(1024 * 1024))
    }

    /// The cap in bytes.
    pub const fn limit(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ReadCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}
