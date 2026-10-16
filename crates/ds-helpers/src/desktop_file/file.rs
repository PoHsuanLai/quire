//! A parsed desktop file: its groups in file order.

use super::error::DesktopFileError;
use super::group::DesktopGroup;
use super::parse::parse_text;
use crate::ReadCap;

/// The group a desktop entry's own keys live in.
const ENTRY_GROUP: &str = "Desktop Entry";

/// The largest desktop file read by default. Real entries are a few KiB and the ones carrying
/// every translation stay well under 100 KiB, so a file over this is not a launcher.
pub const DESKTOP_ENTRY_CAP: ReadCap = ReadCap::kib(256);

/// A `.desktop` file (or any file in the same key file format, such as an icon theme's
/// `index.theme`) as groups of keys. See the module doc for what is skipped and how duplicates
/// are read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopFile {
    pub(super) groups: Vec<DesktopGroup>,
    pub(super) skipped: usize,
}

impl DesktopFile {
    /// The file `text` holds; [`DesktopFileError::TooBig`] when `text` is longer than `cap`
    /// ([`DESKTOP_ENTRY_CAP`] suits a desktop entry).
    pub fn parse(text: &str, cap: ReadCap) -> Result<DesktopFile, DesktopFileError> {
        refuse_over(text.len(), cap)?;
        Ok(parse_text(text))
    }

    /// The file `bytes` hold, with anything that is not UTF-8 replaced; refused over `cap` as
    /// [`DesktopFile::parse`] does.
    pub fn from_bytes(bytes: &[u8], cap: ReadCap) -> Result<DesktopFile, DesktopFileError> {
        refuse_over(bytes.len(), cap)?;
        Ok(parse_text(&String::from_utf8_lossy(bytes)))
    }

    /// The group called `name`, if the file has one.
    pub fn group(&self, name: &str) -> Option<&DesktopGroup> {
        self.groups.iter().find(|group| group.name() == name)
    }

    /// The `[Desktop Entry]` group: the entry itself. Its `Desktop Action` groups are named by its
    /// `Actions` list and found with [`DesktopFile::group`].
    pub fn entry(&self) -> Option<&DesktopGroup> {
        self.group(ENTRY_GROUP)
    }

    /// Every group, in the order the file first names them.
    pub fn groups(&self) -> impl Iterator<Item = &DesktopGroup> {
        self.groups.iter()
    }

    /// How many lines were skipped as neither a comment, a group header nor a `Key=value` pair,
    /// or as a pair before any group.
    pub fn skipped_lines(&self) -> usize {
        self.skipped
    }
}

/// Refuses a text of `size` bytes over `cap`.
fn refuse_over(size: usize, cap: ReadCap) -> Result<(), DesktopFileError> {
    let size = u64::try_from(size).unwrap_or(u64::MAX);
    if size > cap.limit() {
        return Err(DesktopFileError::TooBig { size, cap });
    }
    Ok(())
}
