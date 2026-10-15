//! Finding a COLRv1 colour emoji font file (`Noto-COLRv1.ttf`) across distros and our install dir
//! (FINDINGS "Colour emoji").
//!
//! Our renderers draw colour emoji only from the COLRv1 build of Noto Color Emoji. Fedora ships
//! it; Ubuntu and Arch ship the bitmap (CBDT) build, which draws nothing, so the desktop installer
//! puts `Noto-COLRv1.ttf` in `<prefix>/share/fonts/quire/`. The search is a pure function of the
//! environment values ([`EmojiDirs`]) and a [`FontFiles`] view of the disk; [`Disk`] is the one
//! implementation that touches the real filesystem. A file is chosen only when it lists a `COLR`
//! table, so a CBDT-only `NotoColorEmoji.ttf` is never the colour face, whatever it is called.

use crate::sfnt::table_listed;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The file name the installer and the COLRv1 packages use.
const COLRV1_FILE: &str = "Noto-COLRv1.ttf";

/// The directory under a data dir's `fonts` that the desktop installer fills.
const QUIRE_DIR: &str = "quire";

/// `XDG_DATA_DIRS` when it is unset or empty.
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// Where Fedora's `google-noto-color-emoji-fonts` puts it.
const FEDORA_PATH: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";

/// Where AUR COLRv1 emoji packages put it. Unverified: the package names and paths are a guess.
const AUR_PATHS: [&str; 2] = [
    "/usr/share/fonts/noto-emoji-colrv1/Noto-COLRv1.ttf",
    "/usr/share/fonts/noto/Noto-COLRv1.ttf",
];

/// How deep the file-name search goes below a font directory.
const SCAN_DEPTH: usize = 4;

/// The environment values that say where fonts live, already split.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct EmojiDirs {
    /// `XDG_DATA_DIRS`, or its default.
    data_dirs: Vec<PathBuf>,
    /// `XDG_DATA_HOME`, or `~/.local/share`.
    data_home: Option<PathBuf>,
    /// `HOME`.
    home: Option<PathBuf>,
}

/// What the search asks of the disk.
pub(crate) trait FontFiles {
    /// Whether `path` is a font file that lists a `COLR` table.
    fn is_colour_font(&self, path: &Path) -> bool;
    /// The files below `dir` whose name contains `COLRv1`, in a stable order.
    fn colrv1_named_under(&self, dir: &Path) -> Vec<PathBuf>;
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|text| !text.is_empty())
}

impl EmojiDirs {
    /// From the values of `XDG_DATA_DIRS`, `XDG_DATA_HOME` and `HOME`. Empty counts as unset.
    pub(crate) fn from_vars(
        xdg_data_dirs: Option<&str>,
        xdg_data_home: Option<&str>,
        home: Option<&str>,
    ) -> EmojiDirs {
        let home = non_empty(home).map(PathBuf::from);
        let data_home = non_empty(xdg_data_home)
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|home| home.join(".local/share")));
        let data_dirs = non_empty(xdg_data_dirs)
            .unwrap_or(DEFAULT_DATA_DIRS)
            .split(':')
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .collect();
        EmojiDirs {
            data_dirs,
            data_home,
            home,
        }
    }

    /// From the process environment.
    pub(crate) fn from_env() -> EmojiDirs {
        let var = |name: &str| std::env::var(name).ok();
        EmojiDirs::from_vars(
            var("XDG_DATA_DIRS").as_deref(),
            var("XDG_DATA_HOME").as_deref(),
            var("HOME").as_deref(),
        )
    }

    /// The files that are named for the COLRv1 build, best first: our install dir under each data
    /// dir, the user's, then the distro packages' places.
    pub(crate) fn candidates(&self) -> Vec<PathBuf> {
        let ours = |base: &PathBuf| base.join("fonts").join(QUIRE_DIR).join(COLRV1_FILE);
        let packaged = std::iter::once(FEDORA_PATH)
            .chain(AUR_PATHS)
            .map(PathBuf::from);
        let all: Vec<PathBuf> = self
            .data_dirs
            .iter()
            .chain(self.data_home.iter())
            .map(ours)
            .chain(packaged)
            .collect();
        dedup(all)
    }

    /// The font directories to look through for a file named `…COLRv1…`.
    fn font_dirs(&self) -> Vec<PathBuf> {
        let dirs: Vec<PathBuf> = self
            .data_dirs
            .iter()
            .chain(self.data_home.iter())
            .map(|base| base.join("fonts"))
            .chain(self.home.iter().map(|home| home.join(".fonts")))
            .collect();
        dedup(dirs)
    }

    /// The first usable COLRv1 colour emoji file: a candidate that lists `COLR`, else any file
    /// named for COLRv1 under the font directories that does.
    pub(crate) fn find(&self, files: &impl FontFiles) -> Option<PathBuf> {
        let named = || {
            self.font_dirs()
                .into_iter()
                .flat_map(|dir| files.colrv1_named_under(&dir))
        };
        self.candidates()
            .into_iter()
            .chain(named())
            .find(|path| files.is_colour_font(path))
    }
}

/// `paths` without repeats, first occurrence kept.
fn dedup(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().fold(Vec::new(), |mut kept, path| {
        if !kept.contains(&path) {
            kept.push(path);
        }
        kept
    })
}

/// The real filesystem.
pub(crate) struct Disk;

impl FontFiles for Disk {
    fn is_colour_font(&self, path: &Path) -> bool {
        // The table directory is at the front of the file; a few KiB hold it.
        let mut head = Vec::new();
        let read =
            std::fs::File::open(path).and_then(|file| file.take(4096).read_to_end(&mut head));
        read.is_ok() && table_listed(&head, 0, *b"COLR")
    }

    fn colrv1_named_under(&self, dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        walk(dir, SCAN_DEPTH, &mut found);
        found.sort();
        found
    }
}

/// Add to `found` the font files below `dir` (to `depth` levels) named for COLRv1.
fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            if depth > 0 {
                walk(&path, depth - 1, found);
            }
        } else if named_for_colrv1(&path) {
            found.push(path);
        }
    }
}

/// Whether the file name contains `COLRv1` (any case) and ends in `.ttf` or `.otf`.
fn named_for_colrv1(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    name.contains("colrv1") && (name.ends_with(".ttf") || name.ends_with(".otf"))
}

/// The COLRv1 colour emoji font file on this machine, found as [`EmojiDirs::find`] does with the
/// process environment and the real disk. `None` where the machine has none.
pub fn colrv1_emoji_file() -> Option<PathBuf> {
    EmojiDirs::from_env().find(&Disk)
}

#[cfg(test)]
mod tests {
    use super::{EmojiDirs, FontFiles, named_for_colrv1};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    /// A disk of named files: the colour ones, and the directories' listings.
    #[derive(Default)]
    struct Fake {
        colour: Vec<&'static str>,
        listings: HashMap<&'static str, Vec<&'static str>>,
    }

    impl FontFiles for Fake {
        fn is_colour_font(&self, path: &Path) -> bool {
            self.colour.iter().any(|colour| path == Path::new(colour))
        }
        fn colrv1_named_under(&self, dir: &Path) -> Vec<PathBuf> {
            let key = dir.to_str().unwrap_or_default();
            let files = self.listings.get(key).cloned().unwrap_or_default();
            files.into_iter().map(PathBuf::from).collect()
        }
    }

    fn dirs(data_dirs: Option<&str>, data_home: Option<&str>, home: Option<&str>) -> EmojiDirs {
        EmojiDirs::from_vars(data_dirs, data_home, home)
    }

    fn strs(paths: Vec<PathBuf>) -> Vec<String> {
        paths.iter().map(|p| p.display().to_string()).collect()
    }

    #[test]
    fn candidates_follow_the_data_dirs_then_the_user_then_the_packages() {
        let got = strs(dirs(Some("/opt/a:/opt/b"), None, Some("/home/u")).candidates());
        assert_eq!(
            got,
            [
                "/opt/a/fonts/quire/Noto-COLRv1.ttf",
                "/opt/b/fonts/quire/Noto-COLRv1.ttf",
                "/home/u/.local/share/fonts/quire/Noto-COLRv1.ttf",
                "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf",
                "/usr/share/fonts/noto-emoji-colrv1/Noto-COLRv1.ttf",
                "/usr/share/fonts/noto/Noto-COLRv1.ttf",
            ]
        );
    }

    #[test]
    fn unset_or_empty_variables_take_their_defaults() {
        for unset in [None, Some("")] {
            let got = strs(dirs(unset, unset, Some("/home/u")).candidates());
            assert_eq!(
                got[..3],
                [
                    "/usr/local/share/fonts/quire/Noto-COLRv1.ttf",
                    "/usr/share/fonts/quire/Noto-COLRv1.ttf",
                    "/home/u/.local/share/fonts/quire/Noto-COLRv1.ttf",
                ]
            );
        }
        let no_home = strs(dirs(None, None, None).candidates());
        assert_eq!(no_home.len(), 5, "no home, no user dir: {no_home:?}");
    }

    #[test]
    fn xdg_data_home_replaces_the_default_user_dir_and_repeats_collapse() {
        let got = strs(dirs(Some("/usr/share"), Some("/usr/share"), Some("/home/u")).candidates());
        let ours = "/usr/share/fonts/quire/Noto-COLRv1.ttf";
        assert_eq!(got.iter().filter(|path| *path == ours).count(), 1);
        assert!(!got.iter().any(|path| path.starts_with("/home/u")));
    }

    #[test]
    fn our_install_dir_beats_a_distro_package() {
        let fake = Fake {
            colour: vec![
                "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf",
                "/usr/local/share/fonts/quire/Noto-COLRv1.ttf",
            ],
            ..Fake::default()
        };
        let found = dirs(None, None, Some("/home/u")).find(&fake);
        assert_eq!(
            found,
            Some(PathBuf::from(
                "/usr/local/share/fonts/quire/Noto-COLRv1.ttf"
            ))
        );
    }

    #[test]
    fn fedora_has_it_without_our_install() {
        let fake = Fake {
            colour: vec!["/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf"],
            ..Fake::default()
        };
        assert_eq!(
            dirs(None, None, None).find(&fake),
            Some(PathBuf::from(
                "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf"
            ))
        );
    }

    #[test]
    fn a_file_named_for_colrv1_elsewhere_under_the_font_dirs_is_found() {
        let fake = Fake {
            colour: vec!["/usr/share/fonts/odd/emoji-COLRv1-2.ttf"],
            listings: HashMap::from([(
                "/usr/share/fonts",
                vec!["/usr/share/fonts/odd/emoji-COLRv1-2.ttf"],
            )]),
        };
        assert_eq!(
            dirs(None, None, None).find(&fake),
            Some(PathBuf::from("/usr/share/fonts/odd/emoji-COLRv1-2.ttf"))
        );
    }

    #[test]
    fn a_bitmap_only_file_is_never_chosen_whatever_it_is_called() {
        // The listing offers the file, but it has no COLR table: the fake does not call it colour.
        let fake = Fake {
            colour: vec![],
            listings: HashMap::from([(
                "/usr/share/fonts",
                vec![
                    "/usr/share/fonts/noto/NotoColorEmoji.ttf",
                    "/usr/share/fonts/x/CBDT-COLRv1.ttf",
                ],
            )]),
        };
        assert_eq!(dirs(None, None, Some("/home/u")).find(&fake), None);
    }

    #[test]
    fn only_colrv1_named_font_files_match_the_name_search() {
        assert!(named_for_colrv1(Path::new("/a/Noto-COLRv1.ttf")));
        assert!(named_for_colrv1(Path::new("/a/noto-colrv1-subset.otf")));
        assert!(!named_for_colrv1(Path::new("/a/NotoColorEmoji.ttf")));
        assert!(!named_for_colrv1(Path::new("/a/Noto-COLRv1.txt")));
    }
}
