//! A freedesktop `.desktop` file (Desktop Entry Specification, "Basic format of the file") read
//! from its text: groups, keys, localized keys, the value escapes and the `;` lists.
//!
//! Nothing here reads a file: the text arrives as a `&str` (or bytes) and is refused over the
//! caller's [`crate::ReadCap`], so a core crate reads the file with [`crate::read_text_capped`]
//! and hands it over. A line that is not a comment, a group header or a `Key=value` pair, and a
//! pair before the first group, is skipped and counted ([`DesktopFile::skipped_lines`]); one bad
//! line never costs the entry.
//!
//! Duplicates: a group named twice is one group, and a key given twice keeps its last value, as
//! GLib's key file does (the specification leaves both undefined).
//!
//! `Exec` and `TryExec` come back as the string the file holds after the value escapes; the field
//! codes (`%f`, `%U`, ...) and the quoting of the command line are the caller's.

mod error;
mod escape;
mod file;
mod group;
mod locale;
mod parse;

pub use error::DesktopFileError;
pub use file::{DESKTOP_ENTRY_CAP, DesktopFile};
pub use group::DesktopGroup;
pub use locale::EntryLocale;
