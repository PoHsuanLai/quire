//! The one place quire reads and writes a person's files safely (a settings file, a list of
//! Spaces, a document an app keeps), usable without any UI, bus or runtime.
//!
//! - [`read_capped`] / [`read_text_capped`]: open a regular file without blocking on a FIFO, take
//!   at most the caller's [`ReadCap`], and say apart "there is no such file" ([`ReadError::NotFound`])
//!   from every other failure, so a file that exists but cannot be read is never taken for a
//!   missing one.
//! - [`write_atomic`]: a unique temporary file beside the target, `sync_all`, a rename, and an
//!   fsync of the directory; the target's mode is kept, a symlink is written through to its
//!   target, and two writers of one path in a process take turns.
//! - [`move_aside`]: the damaged file kept under a timestamped `.bad` name before anything is
//!   written in its place.

mod aside;
mod cap;
mod link;
mod lock;
mod read;
mod write;

pub use aside::{UnixSeconds, move_aside};
pub use cap::ReadCap;
pub use read::{ReadError, read_capped, read_text_capped};
pub use write::{NewFileMode, WriteError, write_atomic};
