//! Files on the clipboard: the typed form an app copies and pastes, and its wire formats.
//!
//! Other programs offer files as `text/uri-list` (one `file://` URI per line) and, for a cut,
//! `x-special/gnome-copied-files` (a first line `copy` or `cut`, then the same URIs). A plain
//! text copy of absolute paths or URIs is read too. The URI encoding is written here, by hand:
//! every byte outside the unreserved set and `/` becomes `%XX`, so spaces, non-ASCII names and a
//! literal `%` round-trip.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What the person did to the files: the first line of `x-special/gnome-copied-files`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FileOp {
    /// Copy: a paste makes new files.
    #[default]
    Copy,
    /// Cut: a paste moves them. The system clipboard cannot carry this to another app (see
    /// `CONSUMING.md`); the app keeps the cut set itself.
    Cut,
}

/// Files on the clipboard: an operation and the paths it applies to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ClipboardFiles {
    /// Copy or cut.
    pub op: FileOp,
    /// The files, as absolute paths, in the order copied.
    pub paths: Vec<PathBuf>,
}

impl ClipboardFiles {
    /// `paths` with `op`.
    pub fn new(op: FileOp, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        ClipboardFiles {
            op,
            paths: paths.into_iter().map(Into::into).collect(),
        }
    }

    /// Whether `other` names the same files, in any order. An app that cut files turns a paste of
    /// the same set into a move.
    pub fn same_files(&self, other: &[PathBuf]) -> bool {
        let mut mine: Vec<&Path> = self.paths.iter().map(PathBuf::as_path).collect();
        let mut theirs: Vec<&Path> = other.iter().map(PathBuf::as_path).collect();
        mine.sort_unstable();
        theirs.sort_unstable();
        mine == theirs
    }

    /// Each path as a `file://` URI.
    pub fn uris(&self) -> Vec<String> {
        self.paths.iter().map(|path| file_uri(path)).collect()
    }

    /// The `text/uri-list` form: one URI per line.
    pub fn to_uri_list(&self) -> String {
        self.uris().join("\r\n")
    }

    /// The `x-special/gnome-copied-files` form: `copy` or `cut`, then one URI per line.
    pub fn to_gnome_copied(&self) -> String {
        let op = match self.op {
            FileOp::Copy => "copy",
            FileOp::Cut => "cut",
        };
        std::iter::once(op.to_owned())
            .chain(self.uris())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The `text/plain` form: one path per line.
    pub fn to_text(&self) -> String {
        self.paths
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Read `text/uri-list` as a copy. `None` when it names no local file. Comment lines (`#`)
    /// and URIs that are not local `file://` ones are skipped.
    pub fn from_uri_list(list: &str) -> Option<ClipboardFiles> {
        Self::collect(FileOp::Copy, list.lines())
    }

    /// Read `x-special/gnome-copied-files`: its first line is the operation. `None` when the
    /// first line is neither `copy` nor `cut`, or no local file follows.
    pub fn from_gnome_copied(data: &str) -> Option<ClipboardFiles> {
        let mut lines = data.lines();
        let op = match lines.next()?.trim() {
            "copy" => FileOp::Copy,
            "cut" => FileOp::Cut,
            _ => return None,
        };
        Self::collect(op, lines)
    }

    /// Read plain text as a copy of files: every non-blank line is an absolute path or a
    /// `file://` URI. `None` when any line is neither (ordinary text is not files).
    pub fn from_text(text: &str) -> Option<ClipboardFiles> {
        let paths = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                if line.starts_with("file://") {
                    path_of_uri(line)
                } else {
                    Path::new(line).is_absolute().then(|| PathBuf::from(line))
                }
            })
            .collect::<Option<Vec<_>>>()?;
        (!paths.is_empty()).then_some(ClipboardFiles {
            op: FileOp::Copy,
            paths,
        })
    }

    fn collect<'a>(op: FileOp, lines: impl Iterator<Item = &'a str>) -> Option<ClipboardFiles> {
        let paths: Vec<PathBuf> = lines
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(path_of_uri)
            .collect();
        (!paths.is_empty()).then_some(ClipboardFiles { op, paths })
    }
}

/// The `file://` URI of `path`.
fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for &byte in path.as_os_str().as_encoded_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(char::from(byte));
            }
            _ => {
                uri.push('%');
                uri.extend([byte >> 4, byte & 0xF].map(|nibble| {
                    char::from_digit(u32::from(nibble), 16)
                        .unwrap_or('0')
                        .to_ascii_uppercase()
                }));
            }
        }
    }
    uri
}

/// The path a local `file://` URI names; `None` for another scheme or a host other than this one.
fn path_of_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    path.starts_with('/').then(|| path_from_bytes(decode(path)))
}

/// `%XX` escapes undone; a `%` not followed by two hex digits stays as it is.
fn decode(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        let escaped = (byte == b'%')
            .then(|| bytes.get(at + 1..at + 3))
            .flatten()
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match escaped {
            Some(decoded) => {
                out.push(decoded);
                at += 3;
            }
            None => {
                out.push(byte);
                at += 1;
            }
        }
    }
    out
}

#[cfg(unix)]
fn path_from_bytes(bytes: Vec<u8>) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    PathBuf::from(OsString::from_vec(bytes))
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: Vec<u8>) -> PathBuf {
    PathBuf::from(OsString::from(String::from_utf8_lossy(&bytes).into_owned()))
}

/// Offer `paths` on the system clipboard as `text/uri-list` (arboard's file list).
///
/// The `arboard::Clipboard` that wrote it stays alive for the process: on X11 it owns the
/// selection and answers every paste request. The next write replaces it.
pub(super) fn write_system(paths: &[PathBuf]) -> Result<(), super::ClipboardError> {
    use std::sync::{Mutex, OnceLock, PoisonError};
    static OWNER: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();
    let mut owner = OWNER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if owner.is_none() {
        *owner = Some(arboard::Clipboard::new().map_err(|_| super::ClipboardError::Unavailable)?);
    }
    owner
        .as_mut()
        .ok_or(super::ClipboardError::Unavailable)?
        .set()
        .file_list(paths)
        .map_err(|_| super::ClipboardError::Unavailable)
}

/// The system clipboard's `text/uri-list`, as the list's text (read on the capped worker).
pub(super) fn read_system() -> Option<String> {
    let paths = arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.get().file_list())
        .ok()?;
    Some(ClipboardFiles::new(FileOp::Copy, paths).to_uri_list())
}
