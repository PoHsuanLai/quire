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
//! - [`update_atomic`]: read a file, edit its text and write it back while holding the file's lock
//!   (in this process and, through a sidecar `flock`, between programs), so two read-modify-write
//!   cycles never lose one another's change.
//! - [`move_aside`]: the damaged file kept under a timestamped `.bad` name before anything is
//!   written in its place.

mod aside;
mod cap;
mod flock;
mod link;
mod lock;
mod read;
mod update;
mod write;

pub use aside::{UnixSeconds, move_aside};
pub use cap::ReadCap;
pub use read::{ReadError, read_capped, read_text_capped};
pub use update::{UpdateError, update_atomic};
pub use write::{NewFileMode, WriteError, write_atomic};
