//! "Your changes": the person's own Extension that holds the icon they picked for single file
//! types (design: Extensions, section 6.2, item 1).
//!
//! It is an Extension like any other, `user.YourChanges` in the person's extensions folder,
//! with a `file_icons` entry for its `file-icons/` folder. A picture the person sets for a type
//! is copied there under the type's icon name (`application-pdf.svg`), so the resolver
//! ([`FileIcons`](crate::FileIcons)) finds it first, ahead of the chosen pack and the standard
//! set. There is no second mechanism: removing a change deletes a file.
//!
//! The Settings and Get Info screens that ask for a picture are the shell's; this is the data
//! rules and the writes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ds_helpers::mime::IconName;
use ds_helpers::{
    NewFileMode, ReadCap, ReadError, WriteError, read_capped, read_text_capped, write_atomic,
};

use crate::file_icons::PICTURE_CAP_BYTES;
use crate::search::Search;

/// The id of the Extension that holds the person's changes.
pub const YOUR_CHANGES_ID: &str = "user.YourChanges";

/// What Settings calls it.
pub const YOUR_CHANGES_NAME: &str = "Your changes";

/// The folder of pictures, inside the Extension's folder.
const PICTURES: &str = "file-icons";
const MANIFEST: &str = "extension.toml";
/// The longest icon name accepted (it becomes a file name).
const NAME_MAX: usize = 128;
/// What an SVG icon may not contain (compared in lower case): script, foreign content, and a
/// link to anywhere.
const SVG_REFUSED: [&str; 6] = [
    "<script",
    "<foreignobject",
    "javascript:",
    "href=\"http",
    "href='http",
    "url(http",
];

/// The manifest, written whole: it has nothing the person edits.
const MANIFEST_TEXT: &str = "id = \"user.YourChanges\"\nname = \"Your changes\"\nprotocol = 1\n\
version = \"1.0.0\"\nsummary = \"The icons you chose for single file types\"\n\n\
[[provides]]\ncapability = \"file_icons\"\ndir = \"file-icons\"\n";

/// Whether the Extension may be removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Removal {
    /// It holds no change: Remove may be offered (or it can just stay, empty).
    Allowed,
    /// It holds at least one change: never removable, since removing it would throw them away.
    Blocked,
}

/// A change that could not be made.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ChangeError {
    /// The type's icon name cannot be a file name.
    #[error("{name:?} is not a usable icon name")]
    BadName {
        /// The name as given.
        name: String,
    },
    /// The picture is not a `.svg` or `.png`, or its content is not that picture.
    #[error("{path:?} is not a picture that can be used as an icon")]
    NotAPicture {
        /// The file.
        path: PathBuf,
    },
    /// The picture could not be read (missing, over the cap, not a file).
    #[error(transparent)]
    Read(#[from] ReadError),
    /// A file could not be written.
    #[error(transparent)]
    Write(#[from] WriteError),
    /// A folder could not be made or listed, or a replaced picture removed.
    #[error("{path}: {source}")]
    Folder {
        /// The path.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
}

/// The person's changes, in their data directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YourChanges {
    folder: PathBuf,
}

impl YourChanges {
    /// The changes held under the data root `person` (the shape of `$XDG_DATA_HOME`, as
    /// [`Search::new`] takes it): `<person>/quire/extensions/user.YourChanges`.
    pub fn in_root(person: &Path) -> YourChanges {
        YourChanges {
            folder: person
                .join("quire")
                .join("extensions")
                .join(YOUR_CHANGES_ID),
        }
    }

    /// The changes under a search's personal root; `None` when the person has none.
    pub fn of(search: &Search) -> Option<YourChanges> {
        search.person_root().map(YourChanges::in_root)
    }

    /// The Extension's folder.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    fn pictures(&self) -> PathBuf {
        self.folder.join(PICTURES)
    }

    /// Sets the picture for the type `icon` names: copies `picture` (an `.svg` or `.png`) into
    /// the Extension's `file-icons/` as `<icon name>.<ext>`, replacing the earlier one, and writes
    /// the Extension's `extension.toml` if it is not there yet. Each file is written whole or not
    /// at all. The Extension first appears in the next scan.
    pub fn set(&self, icon: &IconName, picture: &Path) -> Result<(), ChangeError> {
        let name = checked_name(icon.specific())?;
        let format = Format::of(picture).ok_or_else(|| not_a_picture(picture))?;
        let bytes = read_capped(picture, ReadCap::bytes(PICTURE_CAP_BYTES))?;
        if !format.holds(&bytes) {
            return Err(not_a_picture(picture));
        }
        let dir = self.pictures();
        fs::create_dir_all(&dir).map_err(|source| folder_error(&dir, source))?;
        write_atomic(&dir.join(format.file(name)), &bytes, NewFileMode::Umask)?;
        // One picture per type: the other format's file, if there, is the one replaced.
        remove_if_there(&dir.join(format.other().file(name)))?;
        self.write_manifest()
    }

    /// Takes the change for the type `icon` names away, so it has the chosen pack's picture or
    /// the standard one again. A type with no change is left as it is.
    pub fn clear(&self, icon: &IconName) -> Result<(), ChangeError> {
        let name = checked_name(icon.specific())?;
        let dir = self.pictures();
        [Format::Svg, Format::Png]
            .into_iter()
            .map(|format| dir.join(format.file(name)))
            .try_for_each(|path| remove_if_there(&path))
    }

    /// The icon names that have a change, sorted.
    pub fn changed(&self) -> Result<Vec<String>, ChangeError> {
        let dir = self.pictures();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(folder_error(&dir, source)),
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| Format::of(path).is_some())
            .filter_map(|path| path.file_stem()?.to_str().map(str::to_owned))
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Whether Remove may be offered: never while a change is held. A folder that cannot be
    /// read counts as holding changes.
    pub fn removal(&self) -> Removal {
        match self.changed() {
            Ok(names) if names.is_empty() => Removal::Allowed,
            _ => Removal::Blocked,
        }
    }

    /// Writes `extension.toml` unless it already says exactly this.
    fn write_manifest(&self) -> Result<(), ChangeError> {
        let path = self.folder.join(MANIFEST);
        let current = read_text_capped(&path, ReadCap::kib(4)).ok();
        if current.as_deref() == Some(MANIFEST_TEXT) {
            return Ok(());
        }
        write_atomic(&path, MANIFEST_TEXT.as_bytes(), NewFileMode::Umask)?;
        Ok(())
    }
}

/// A picture format an icon may be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Svg,
    Png,
}

impl Format {
    fn of(path: &Path) -> Option<Format> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "svg" => Some(Format::Svg),
            "png" => Some(Format::Png),
            _ => None,
        }
    }

    fn other(self) -> Format {
        match self {
            Format::Svg => Format::Png,
            Format::Png => Format::Svg,
        }
    }

    /// The file name `name` has in this format.
    fn file(self, name: &str) -> String {
        match self {
            Format::Svg => format!("{name}.svg"),
            Format::Png => format!("{name}.png"),
        }
    }

    /// Whether `bytes` are a picture of this format: the PNG signature, or text with an `<svg`
    /// root that carries no script, no foreign content and no link to anywhere.
    fn holds(self, bytes: &[u8]) -> bool {
        match self {
            Format::Png => bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Format::Svg => std::str::from_utf8(bytes).is_ok_and(|text| {
                let text = text.to_ascii_lowercase();
                text.contains("<svg") && SVG_REFUSED.iter().all(|bad| !text.contains(bad))
            }),
        }
    }
}

/// Removes the file at `path`; one that is not there is already gone.
fn remove_if_there(path: &Path) -> Result<(), ChangeError> {
    match fs::remove_file(path) {
        Err(source) if source.kind() != io::ErrorKind::NotFound => Err(folder_error(path, source)),
        _ => Ok(()),
    }
}

fn not_a_picture(path: &Path) -> ChangeError {
    ChangeError::NotAPicture {
        path: path.to_path_buf(),
    }
}

fn folder_error(path: &Path, source: io::Error) -> ChangeError {
    ChangeError::Folder {
        path: path.to_path_buf(),
        source,
    }
}

/// `name` if it can be a file name here: letters, digits and `- _ . +`, not starting with a dot,
/// at most [`NAME_MAX`] characters.
fn checked_name(name: &str) -> Result<&str, ChangeError> {
    let fits = !name.is_empty()
        && name.len() <= NAME_MAX
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+'));
    if fits {
        Ok(name)
    } else {
        Err(ChangeError::BadName {
            name: name.to_owned(),
        })
    }
}
