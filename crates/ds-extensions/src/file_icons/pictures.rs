//! The pictures of one pack folder, read once into a table keyed by name.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// The largest picture a pack may carry: 1 MiB. A larger file is left out and its type falls
/// through to the next layer.
pub const PICTURE_CAP_BYTES: u64 = 1024 * 1024;

/// The most pictures read from one folder.
const PICTURES_MAX: usize = 8192;

/// A picture's format, which decides which file wins when a name has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Format {
    /// A vector picture; preferred.
    Svg,
    /// A bitmap.
    Png,
}

impl Format {
    fn of(extension: Option<&OsStr>) -> Option<Format> {
        match extension?.to_str()?.to_ascii_lowercase().as_str() {
            "svg" => Some(Format::Svg),
            "png" => Some(Format::Png),
            _ => None,
        }
    }
}

/// The pictures in a pack folder: `application-pdf.svg` is the picture named `application-pdf`.
/// Built once; the lookups after it read no file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pictures {
    by_name: HashMap<String, PathBuf>,
}

impl Pictures {
    /// The pictures in `dir`: `.svg` and `.png` files, each under the cap, each really inside the
    /// folder (a link that leads out is left out). A folder that cannot be read has none.
    pub fn read(dir: &Path) -> Pictures {
        let Ok(home) = fs::canonicalize(dir) else {
            return Pictures::default();
        };
        let Ok(entries) = fs::read_dir(&home) else {
            return Pictures::default();
        };
        let mut found: HashMap<String, (Format, PathBuf)> = HashMap::new();
        for entry in entries.flatten().take(PICTURES_MAX) {
            let path = entry.path();
            let (Some(format), Some(stem)) = (
                Format::of(path.extension()),
                path.file_stem().and_then(OsStr::to_str),
            ) else {
                continue;
            };
            let Ok(real) = fs::canonicalize(&path) else {
                continue;
            };
            let usable = real.starts_with(&home)
                && fs::metadata(&real)
                    .is_ok_and(|meta| meta.is_file() && meta.len() <= PICTURE_CAP_BYTES);
            let better = found.get(stem).is_none_or(|(held, _)| format < *held);
            if usable && better {
                found.insert(stem.to_owned(), (format, real));
            }
        }
        Pictures {
            by_name: found
                .into_iter()
                .map(|(name, (_, path))| (name, path))
                .collect(),
        }
    }

    /// The picture named `name`, if the pack has one.
    pub fn get(&self, name: &str) -> Option<&Path> {
        self.by_name.get(name).map(PathBuf::as_path)
    }

    /// How many pictures there are.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}
