//! A settings file's bytes as text: the one place a file is read, so a file that is not UTF-8 is
//! told apart from one that is missing.

use std::path::Path;

/// What reading a settings file found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileText {
    /// The file's text.
    Text(String),
    /// No file, or one that cannot be read: the program looks as it did on first run.
    Missing,
    /// A file that is not UTF-8, with why. A watch keeps the last good value.
    NotText(String),
}

/// The text of the file at `path`.
pub(crate) fn file_text(path: &Path) -> FileText {
    match std::fs::read(path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => FileText::Text(text),
            Err(error) => FileText::NotText(error.to_string()),
        },
        Err(_) => FileText::Missing,
    }
}
