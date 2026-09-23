//! A whole program's settings schema: what a program ships, and what the Settings app
//! discovers (design/22-SETTINGS.md sections 9.1-9.2).

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::key::KeySpec;

/// The directory name a schema's owning program installs under: `quire`, `sill`, `mailo`. The
/// derive reads it off the first path segment of `#[settings(file = "...")]`
/// (`crates/ds-settings-derive/src/gen_struct.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(pub String);

/// The settings file a key lives in, relative to its app: `"quire/appearance.toml"`,
/// `"sill/settings.toml"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FilePath(pub String);

/// One program's whole schema, as `<app-id>.settings.toml` ships it (section 9.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub app: AppId,
    pub file: FilePath,
    pub version: u16,
    /// One `[[key]]` table per entry, TOML's array-of-tables (section 9.2).
    pub key: Vec<KeySpec>,
}

/// Why a candidate `*.settings.toml` file under a discovery directory was not returned by
/// [`discover`]. Not an error the caller must handle: discovery logs it and moves on (section
/// 9.2, "a schema without a program... is skipped with a warning").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stale {
    /// The file could not be read at all.
    Unreadable(String),
    /// The file's bytes are not this schema's TOML shape.
    Malformed(String),
}

impl Schema {
    /// The TOML text this schema writes as `<app-id>.settings.toml`.
    pub fn to_toml(&self) -> String {
        toml::to_string(self)
            .expect("Schema fields are all TOML-representable: strings, numbers, enums and Vec")
    }

    /// The schema `text` holds, or why it does not parse.
    pub fn from_toml(text: &str) -> Result<Schema, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// Writes `<app-id>.settings.toml` into `dir` (creating it if needed) by atomic
    /// temp-file-plus-rename, the same mechanism every settings file in this workspace uses
    /// (design/22-SETTINGS.md section 2).
    pub fn write_to(&self, dir: &Path) -> io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let target = dir.join(format!("{}.settings.toml", self.app.0));
        let temp = dir.join(format!(".{}.settings.toml.tmp", self.app.0));
        std::fs::write(&temp, self.to_toml())?;
        std::fs::rename(&temp, &target)?;
        Ok(target)
    }
}

/// Every valid schema found directly under each of `dirs` (section 9.3: "discover every
/// `*.settings.toml` under `$XDG_DATA_DIRS/quire/settings/`..."), in directory order and then
/// file name order within a directory. A directory that does not exist is skipped silently (a
/// normal `$XDG_DATA_DIRS` entry with nothing installed yet); a file that exists but will not
/// parse is skipped with a warning on stderr (section 9.2, "a schema without a program... is
/// skipped with a warning") — [`discover_reporting`] returns the same list plus the reasons,
/// for a caller (or a test) that wants them instead of a print.
pub fn discover(dirs: &[PathBuf]) -> Vec<Schema> {
    discover_reporting(dirs)
        .into_iter()
        .filter_map(|(path, result)| match result {
            Ok(schema) => Some(schema),
            Err(stale) => {
                eprintln!(
                    "quire settings: skipping stale schema {}: {stale:?}",
                    path.display()
                );
                None
            }
        })
        .collect()
}

/// [`discover`]'s work, without the printing: every candidate file under `dirs`, and whether it
/// parsed.
pub fn discover_reporting(dirs: &[PathBuf]) -> Vec<(PathBuf, Result<Schema, Stale>)> {
    let mut found = Vec::new();
    for dir in dirs {
        let Ok(mut entries) = std::fs::read_dir(dir).map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| name.ends_with(".settings.toml"))
                })
                .collect::<Vec<_>>()
        }) else {
            continue;
        };
        entries.sort();
        for path in entries {
            let parsed = match std::fs::read_to_string(&path) {
                Ok(text) => Schema::from_toml(&text).map_err(Stale::Malformed),
                Err(e) => Err(Stale::Unreadable(e.to_string())),
            };
            found.push((path, parsed));
        }
    }
    found
}

/// `$XDG_DATA_HOME/quire/settings` then every `$XDG_DATA_DIRS`-listed `<dir>/quire/settings`,
/// in that order (`$XDG_DATA_HOME` first so a user's own install shadows a system one), the
/// directories [`discover`] scans (section 9.2). Falls back to the XDG defaults
/// (`~/.local/share`, `/usr/local/share:/usr/share`) when the variable is unset, per the XDG
/// base directory specification.
pub fn data_dirs() -> Vec<PathBuf> {
    data_dirs_from(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("XDG_DATA_DIRS"),
        std::env::var_os("HOME"),
    )
}

fn data_dirs_from(
    xdg_data_home: Option<std::ffi::OsString>,
    xdg_data_dirs: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Vec<PathBuf> {
    let home_dir = xdg_data_home
        .map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".local/share")));
    let dirs_list = xdg_data_dirs
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());
    home_dir
        .into_iter()
        .chain(
            dirs_list
                .split(':')
                .filter(|s| !s.is_empty())
                .map(PathBuf::from),
        )
        .map(|base| base.join("quire").join("settings"))
        .collect()
}

/// `--write-schema <dir>` (section 9.2: "developers run `cargo run -p <app> -- --write-schema
/// <dir>` for a local install"), a helper a program's own `main` wires into its argument
/// parsing. Returns `Ok(true)` (and has written the file) when `args` asked for it, `Ok(false)`
/// when the flag was absent so the caller's normal startup should continue.
pub fn maybe_write_schema(schema: &Schema, args: &[String]) -> io::Result<bool> {
    let Some(position) = args.iter().position(|arg| arg == "--write-schema") else {
        return Ok(false);
    };
    let Some(dir) = args.get(position + 1) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--write-schema needs a directory argument",
        ));
    };
    schema.write_to(Path::new(dir))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{Stale, data_dirs_from, discover, discover_reporting, maybe_write_schema};
    use crate::schema::{AppId, Exposure, FilePath, KeyKind, KeyPath, Label, Page, Schema};
    use std::ffi::OsString;

    fn sample() -> Schema {
        Schema {
            app: AppId("quire".to_owned()),
            file: FilePath("quire/appearance.toml".to_owned()),
            version: 1,
            key: vec![crate::schema::KeySpec {
                path: KeyPath("appearance.theme".to_owned()),
                kind: KeyKind::Segmented {
                    variants: vec!["system".to_owned(), "light".to_owned(), "dark".to_owned()],
                },
                default: toml::Value::String("system".to_owned()),
                label: Label("Theme".to_owned()),
                help: crate::schema::Help(String::new()),
                page: Page::Appearance,
                section: crate::schema::Section("Appearance".to_owned()),
                exposure: Exposure::Basic,
                deprecated: crate::schema::Deprecated::No,
            }],
        }
    }

    #[test]
    fn a_schema_round_trips_through_toml() {
        let schema = sample();
        let back = Schema::from_toml(&schema.to_toml()).unwrap();
        assert_eq!(back, schema);
    }

    #[test]
    fn write_to_then_discover_finds_it() {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{}",
            std::process::id(),
            "write-then-discover"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let schema = sample();
        schema.write_to(&dir).unwrap();
        let found = discover(std::slice::from_ref(&dir));
        assert_eq!(found, vec![schema]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_nonexistent_directory_is_skipped_silently() {
        let dir = std::env::temp_dir().join("ds-settings-schema-test-does-not-exist");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(discover(&[dir]), Vec::new());
    }

    #[test]
    fn a_malformed_file_is_reported_stale_not_returned() {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{}",
            std::process::id(),
            "malformed"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("broken.settings.toml"), "not = [valid").unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());
        let reported = discover_reporting(std::slice::from_ref(&dir));
        assert_eq!(reported.len(), 1);
        assert!(matches!(reported[0].1, Err(Stale::Malformed(_))));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn xdg_data_home_comes_before_xdg_data_dirs() {
        let dirs = data_dirs_from(
            Some(OsString::from("/xdg-data-home")),
            Some(OsString::from("/a:/b")),
            Some(OsString::from("/home/ada")),
        );
        assert_eq!(
            dirs,
            vec![
                std::path::PathBuf::from("/xdg-data-home/quire/settings"),
                std::path::PathBuf::from("/a/quire/settings"),
                std::path::PathBuf::from("/b/quire/settings"),
            ]
        );
    }

    #[test]
    fn xdg_data_home_falls_back_to_home_local_share() {
        let dirs = data_dirs_from(
            None,
            Some(OsString::from("/a")),
            Some(OsString::from("/home/ada")),
        );
        assert_eq!(
            dirs[0],
            std::path::PathBuf::from("/home/ada/.local/share/quire/settings")
        );
    }

    #[test]
    fn maybe_write_schema_writes_only_when_the_flag_is_present() {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{}",
            std::process::id(),
            "cli"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let schema = sample();

        let absent = maybe_write_schema(&schema, &["quire".to_owned()]).unwrap();
        assert!(!absent);
        assert!(!dir.exists());

        let present = maybe_write_schema(
            &schema,
            &[
                "quire".to_owned(),
                "--write-schema".to_owned(),
                dir.to_string_lossy().into_owned(),
            ],
        )
        .unwrap();
        assert!(present);
        assert!(dir.join("quire.settings.toml").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn maybe_write_schema_without_a_directory_argument_is_an_error() {
        let schema = sample();
        let args = vec!["quire".to_owned(), "--write-schema".to_owned()];
        assert!(maybe_write_schema(&schema, &args).is_err());
    }
}
