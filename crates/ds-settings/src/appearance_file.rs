//! Reading and writing `appearance.toml` (design/22-SETTINGS.md section 2): one instance of
//! the generic [`crate::file`] API, with an atomic temp-and-rename write and a read in which a
//! damaged preference is the first run.

use crate::error::SettingsError;
use crate::file::{FileName, Format, Settings};
use crate::settings::AppearanceFile;
use std::path::Path;

/// The file's name inside the program's config directory.
pub const FILE_NAME: &str = "appearance.toml";

/// `appearance.toml`, holding an [`AppearanceFile`].
pub const APPEARANCE: Settings<AppearanceFile> = Settings::new(FileName(FILE_NAME), Format::Toml);

/// The stored settings, or the defaults when there are none or they cannot be read.
///
/// Not an error the user needs to see: a missing or damaged file means the surface looks as it
/// did on first run, and a bad value costs only its own field.
pub fn load(dir: &Path) -> AppearanceFile {
    APPEARANCE.load(dir)
}

/// Write `file` to `dir`, creating the directory if needed, through a temporary file and a
/// rename ([`crate::file::save`]).
pub fn save(dir: &Path, file: &AppearanceFile) -> Result<(), SettingsError> {
    APPEARANCE.save(dir, file)
}

#[cfg(test)]
mod tests {
    use super::{FILE_NAME, load, save};
    use crate::settings::{AppearanceFile, IconDarkVariant, PlateGlyphPolicy};
    use crate::test_dir::TempDir;
    use crate::units::Percent;
    use ds::{Accent, Look, Motion, Theme, Warmth};
    use std::path::Path;

    fn entries(dir: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|e| panic!("{e}"))
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    fn with(edit: impl FnOnce(&mut AppearanceFile)) -> AppearanceFile {
        let mut file = AppearanceFile::default();
        edit(&mut file);
        file
    }

    #[test]
    fn a_file_round_trips() {
        let dir = TempDir::new();
        let fresh = dir.path().join("quire");
        let cases = [
            AppearanceFile::default(),
            with(|f| {
                f.appearance.theme = Theme::Dark;
                f.appearance.accent = Accent::Violet;
                f.appearance.motion_level = Motion::Reduced;
            }),
            with(|f| {
                f.appearance.look = Look::Candy;
                f.appearance.warmth = Warmth::Paper;
                f.appearance.material_tint_alpha = Percent(64);
                f.icons.plate_glyph_colour_policy = PlateGlyphPolicy::ForceInk;
                f.icons.dark_mode_variant = IconDarkVariant::Adaptive;
            }),
        ];
        for file in cases {
            save(&fresh, &file).unwrap_or_else(|e| panic!("{file:?}: {e}"));
            assert_eq!(load(&fresh), file, "{file:?}");
            // The rename is the whole of the write: a temp file left beside the real one
            // is a crash that did not finish, and this directory had no other files.
            assert_eq!(entries(&fresh), [FILE_NAME], "{file:?}");
        }
    }

    #[test]
    fn a_missing_file_is_the_first_run() {
        let dir = TempDir::new();
        assert_eq!(load(dir.path()), AppearanceFile::default());
    }

    #[test]
    fn garbage_bytes_are_the_first_run() {
        let dir = TempDir::new();
        let path = dir.path().join(FILE_NAME);
        const CASES: &[(&str, &[u8])] = &[
            ("empty", b""),
            ("prose", b"not toml [[[ = ="),
            ("binary", &[0xff, 0xfe, b'[']),
        ];
        for &(name, bytes) in CASES {
            std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(load(dir.path()), AppearanceFile::default(), "{name}");
        }
    }

    #[test]
    fn a_bad_value_costs_only_its_own_field() {
        // Every row also sets a field to something other than its default, so a loader that
        // threw the whole file away on the bad value would fail it.
        let cases: &[(&str, &str, AppearanceFile)] = &[
            (
                "unknown theme keeps the motion",
                "[appearance]\ntheme = \"sepia\"\nmotion_level = \"reduced\"\n",
                with(|f| f.appearance.motion_level = Motion::Reduced),
            ),
            (
                "unknown motion keeps the theme",
                "[appearance]\ntheme = \"dark\"\nmotion_level = \"wild\"\n",
                with(|f| f.appearance.theme = Theme::Dark),
            ),
            (
                "a number where a word belongs keeps the accent",
                "[appearance]\nlook = 7\naccent = \"green\"\n",
                with(|f| f.appearance.accent = Accent::Green),
            ),
            (
                "an out-of-range percent is clamped",
                "[appearance]\nmaterial_tint_alpha = 255\n",
                with(|f| f.appearance.material_tint_alpha = Percent(100)),
            ),
            (
                "a negative percent is the default and keeps the theme",
                "[appearance]\nmaterial_tint_alpha = -4\ntheme = \"light\"\n",
                with(|f| f.appearance.theme = Theme::Light),
            ),
            (
                "a missing table is that table's defaults",
                "[icons]\nplate_inset_percent = 60\n",
                with(|f| f.icons.plate_inset_percent = Percent(60)),
            ),
            (
                "a table that is not a table is its defaults",
                "appearance = 3\n[icons]\ndark_mode_variant = \"adaptive\"\n",
                with(|f| f.icons.dark_mode_variant = IconDarkVariant::Adaptive),
            ),
        ];
        let dir = TempDir::new();
        let path = dir.path().join(FILE_NAME);
        for (name, text, want) in cases {
            std::fs::write(&path, text).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(&load(dir.path()), want, "{name}: {text}");
        }
    }

    #[test]
    fn unknown_keys_survive_a_round_trip() {
        let dir = TempDir::new();
        let path = dir.path().join(FILE_NAME);
        let text = "version = 1\nfuture = \"kept\"\n[appearance]\ntheme = \"dark\"\npuppy = true\n\
                    [icons]\nsparkle = 3\n[later]\nanswer = 42\n";
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("{e}"));
        let read = load(dir.path());
        assert_eq!(read.appearance.theme, Theme::Dark);
        save(dir.path(), &read).unwrap_or_else(|e| panic!("{e}"));
        let written = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        let tree: toml::Table = toml::from_str(&written).unwrap_or_else(|e| panic!("{e}"));
        let at = |table: &str, key: &str| {
            tree.get(table)
                .and_then(|t| t.get(key))
                .cloned()
                .unwrap_or_else(|| panic!("{table}.{key} dropped from {written}"))
        };
        assert_eq!(at("appearance", "puppy"), toml::Value::Boolean(true));
        assert_eq!(at("icons", "sparkle"), toml::Value::Integer(3));
        assert_eq!(at("later", "answer"), toml::Value::Integer(42));
        assert_eq!(
            tree.get("future"),
            Some(&toml::Value::String("kept".to_owned())),
            "{written}"
        );
    }
}
