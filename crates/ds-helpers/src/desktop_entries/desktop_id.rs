//! The id of a desktop entry, built from its place under an `applications` folder.

use std::fmt;
use std::path::Path;

use thiserror::Error;

/// The extension a desktop entry has.
const EXTENSION: &str = "desktop";

/// Why a text is not a desktop id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum DesktopIdError {
    /// It does not end in `.desktop`.
    #[error("An app id ends in \u{201c}.desktop\u{201d}.")]
    Suffix,
    /// Nothing comes before `.desktop`.
    #[error("An app id needs a name before \u{201c}.desktop\u{201d}.")]
    Empty,
    /// It holds a `/`, or white space.
    #[error("An app id has no slash or spaces.")]
    Shape,
}

/// A desktop file ID (Desktop Entry Specification, "Desktop File ID"): the file's path under its
/// `applications` folder with each `/` made a `-`, so `kde/foo.desktop` is `kde-foo.desktop`. The
/// text keeps the `.desktop` suffix; [`DesktopId::stem`] is the id without it, the form a window's
/// app id is compared with.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DesktopId(String);

impl DesktopId {
    /// The id of the entry at `relative`, its path below an `applications` folder. None when it
    /// is not a `*.desktop` file or its name is empty.
    pub fn from_relative(relative: &Path) -> Option<DesktopId> {
        if relative.extension().is_none_or(|ext| ext != EXTENSION) {
            return None;
        }
        let stem = relative
            .with_extension("")
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("-");
        (!stem.is_empty()).then(|| DesktopId(format!("{stem}.{EXTENSION}")))
    }

    /// The id written as text, `foo.desktop` or `foo`; none when nothing is left of the name.
    pub fn parse(text: &str) -> Option<DesktopId> {
        let stem = text.strip_suffix(".desktop").unwrap_or(text);
        (!stem.is_empty()).then(|| DesktopId(format!("{stem}.{EXTENSION}")))
    }

    /// `text`, when it ends in `.desktop` after a name, with no slash or white space: the strict
    /// reading, for ids that `mimeapps.list` and `mimeinfo.cache` name. [`DesktopId::parse`] is the
    /// lenient one.
    pub fn new(text: &str) -> Result<DesktopId, DesktopIdError> {
        let Some(stem) = text.strip_suffix(".desktop") else {
            return Err(DesktopIdError::Suffix);
        };
        if stem.is_empty() {
            return Err(DesktopIdError::Empty);
        }
        if text.contains('/') || text.contains(char::is_whitespace) {
            return Err(DesktopIdError::Shape);
        }
        Ok(DesktopId(text.to_owned()))
    }

    /// The id with its `.desktop` suffix.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id without the suffix: `kde-foo`.
    pub fn stem(&self) -> &str {
        self.0.strip_suffix(".desktop").unwrap_or(&self.0)
    }
}

impl fmt::Display for DesktopId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
