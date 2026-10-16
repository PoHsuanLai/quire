//! A settings file's bytes as text: the one place a settings file is read, so a file that is
//! missing is told apart from one that is there and cannot be used.

use ds_helpers::{ReadCap, ReadError, read_text_capped};
use std::path::Path;

/// The most a settings file may hold. Hand-edited settings are a few KiB; a megabyte is already
/// a mistake, and a bigger file would stall startup in the key-by-key reader.
pub(crate) const SETTINGS_CAP: ReadCap = ReadCap::mib(1);

/// What reading a settings file found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileText {
    /// The file's text.
    Text(String),
    /// No file at the path: the program looks as it did on first run.
    Missing,
    /// A file that is there and cannot be used (not text, too big, not a regular file, refused
    /// by the system), with why. The file is the person's: it is neither replaced by defaults nor
    /// written over. A watch keeps the last good value.
    Unusable(String),
}

/// The text of the file at `path`, at most `cap` bytes of it.
pub(crate) fn file_text(path: &Path, cap: ReadCap) -> FileText {
    match read_text_capped(path, cap) {
        Ok(text) => FileText::Text(text),
        Err(error) if error.is_missing() => FileText::Missing,
        Err(error) => FileText::Unusable(reason(&error)),
    }
}

/// Why a read was refused, as a short phrase.
fn reason(error: &ReadError) -> String {
    match error {
        ReadError::NotRegular { .. } => "not a regular file".to_owned(),
        ReadError::TooBig { cap, .. } => format!("too large (over {cap})"),
        ReadError::NotText { source, .. } => format!("not text: {source}"),
        ReadError::Io { source, .. } => format!("cannot be read: {source}"),
        other => other.to_string(),
    }
}
