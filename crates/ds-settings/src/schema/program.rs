//! A whole program's settings schema: what a program ships, and what the Settings app
//! discovers (design/22-SETTINGS.md sections 9.1-9.2).

use std::io;
use std::path::{Path, PathBuf};

use ds_helpers::{NewFileMode, ReadCap, read_text_capped, write_atomic};
use serde::{Deserialize, Serialize};

use super::agent::never_settable_violation;
use super::key::KeySpec;
use crate::xdg;

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

/// The top-level tables of a program's settings file that the program owns but that are not
/// settings (an engine list, a probe table): the Settings app and detent's unknown-key check
/// leave every key under them alone (design/22-SETTINGS.md section 9.2). Empty for most files,
/// and then not written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ForeignTables(pub Vec<String>);

impl ForeignTables {
    /// Whether `path` (a dotted key path: `engines.local.cmd`) sits under one of these tables.
    pub fn holds(&self, path: &str) -> bool {
        let first = path.split('.').next().unwrap_or(path);
        self.0.iter().any(|table| table == first)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One program's whole schema, as `<app-id>.settings.toml` ships it (section 9.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub app: AppId,
    pub file: FilePath,
    pub version: u16,
    /// `foreign = ["engines", "probe"]`: the file's tables that are the program's own data,
    /// not settings. Absent in the TOML means none.
    #[serde(default, skip_serializing_if = "ForeignTables::is_empty")]
    pub foreign: ForeignTables,
    /// One `[[key]]` table per entry, TOML's array-of-tables (section 9.2).
    pub key: Vec<KeySpec>,
}

impl Schema {
    /// The TOML text this schema writes as `<app-id>.settings.toml`.
    pub fn to_toml(&self) -> String {
        toml::to_string(self)
            .expect("Schema fields are all TOML-representable: strings, numbers, enums and Vec")
    }

    /// The schema `text` holds, or why it does not parse. A key under an
    /// [`AGENT_NEVER_SETTABLE`](super::AGENT_NEVER_SETTABLE) prefix that is marked
    /// `agent = "settable"` is an error, so a schema file any program installs cannot make
    /// the companion's own limits, computer use or a lock agent-settable.
    pub fn from_toml(text: &str) -> Result<Schema, String> {
        let schema: Schema = toml::from_str(text).map_err(|e| e.to_string())?;
        match never_settable_violation(&schema.key) {
            Some(key) => Err(format!(
                "key {} is under a never-agent-settable prefix but is marked agent = \"settable\"",
                key.path.0
            )),
            None => Ok(schema),
        }
    }

    /// Writes `<app-id>.settings.toml` into `dir` (creating it if needed) through
    /// [`ds_helpers::write_atomic`], the same mechanism every settings file in this workspace
    /// uses (design/22-SETTINGS.md section 2).
    pub fn write_to(&self, dir: &Path) -> io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let target = dir.join(format!("{}.settings.toml", self.app.0));
        write_atomic(&target, self.to_toml().as_bytes(), NewFileMode::Umask)
            .map_err(|error| error.source)?;
        Ok(target)
    }
}

/// The most a schema file may hold: a few KiB for a program with a hundred keys.
const SCHEMA_CAP: ReadCap = ReadCap::kib(256);

/// The suffix of a schema file's name; what precedes it is the program it describes.
const SCHEMA_SUFFIX: &str = ".settings.toml";

/// Every valid schema found directly under each of `dirs` (section 9.3: "discover every
/// `*.settings.toml` under `$XDG_DATA_DIRS/quire/settings/`..."), in directory order and then
/// file name order within a directory. A directory that does not exist is skipped silently (a
/// normal `$XDG_DATA_DIRS` entry with nothing installed yet); a file that exists but will not
/// parse is skipped with a warning on stderr (section 9.2, "a schema without a program... is
/// skipped with a warning").
///
/// A schema is admitted only for what its location says it is:
///
/// - its file is named `<app>.settings.toml` for the `app` it declares, and the settings file it
///   names (`file`) is inside that app's own directory, so a schema cannot describe another
///   program's file;
/// - the first schema found for an app wins, so the order of `dirs` is the order of trust:
///   [`data_dirs`] lists the system directories before the user's own, and a schema in the
///   user's directory can add a program but never replace an installed one's;
/// - it is no larger than 256 KiB and a regular file.
///
/// The `agent = "settable"` marks of the keys it returns are therefore those of a schema this
/// function admitted, read through [`KeySpec::agent`].
pub fn discover(dirs: &[PathBuf]) -> Vec<Schema> {
    let mut found: Vec<Schema> = Vec::new();
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
                            .is_some_and(|name| name.ends_with(SCHEMA_SUFFIX))
                })
                .collect::<Vec<_>>()
        }) else {
            continue;
        };
        entries.sort();
        for path in entries {
            match admit(&path, &found) {
                Ok(schema) => found.push(schema),
                Err(why) => eprintln!(
                    "quire settings: skipping stale schema {}: {why}",
                    path.display()
                ),
            }
        }
    }
    found
}

/// The schema at `path`, if it may join `found`.
fn admit(path: &Path, found: &[Schema]) -> Result<Schema, String> {
    let text = read_text_capped(path, SCHEMA_CAP).map_err(|error| error.to_string())?;
    let schema = Schema::from_toml(&text)?;
    let app = schema.app.0.as_str();
    let named_for = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(SCHEMA_SUFFIX));
    if named_for != Some(app) {
        return Err(format!("the file is not named for its program, {app}"));
    }
    if schema.file.0.split('/').next() != Some(app) {
        return Err(format!(
            "it names {}, which is not inside {app}'s own directory",
            schema.file.0
        ));
    }
    if found.iter().any(|earlier| earlier.app == schema.app) {
        return Err(format!(
            "{app} already has a schema in an earlier directory"
        ));
    }
    Ok(schema)
}

/// Every `$XDG_DATA_DIRS`-listed `<dir>/quire/settings`, then `$XDG_DATA_HOME/quire/settings`
/// last, the directories [`discover`] scans (section 9.2). The user's directory comes last
/// because [`discover`] keeps the first schema it finds for a program: a user's own install can
/// add a program but never shadow an installed one (and so never change which of its keys an
/// agent may set). Falls back to the XDG defaults
/// (`~/.local/share`, `/usr/local/share:/usr/share`) when the variable is unset, per the XDG
/// base directory specification.
pub fn data_dirs() -> Vec<PathBuf> {
    data_dirs_from(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("XDG_DATA_DIRS"),
        std::env::var_os("HOME"),
    )
}

/// [`data_dirs`] with the environment handed in: `xdg_data_home` (else `home`'s `.local/share`)
/// and `xdg_data_dirs` (else `/usr/local/share:/usr/share`), so a caller that owns its
/// directories, or a test, names them itself. The XDG rules (`crate::xdg`) apply: an empty or
/// relative value is ignored.
pub fn data_dirs_from(
    xdg_data_home: Option<std::ffi::OsString>,
    xdg_data_dirs: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Vec<PathBuf> {
    xdg::search_dirs(xdg_data_dirs.as_deref())
        .into_iter()
        .chain(xdg::base_dir(
            xdg_data_home.as_deref(),
            home.as_deref(),
            xdg::DATA_HOME_UNDER_HOME,
        ))
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
    use super::{data_dirs_from, discover, maybe_write_schema};
    use crate::schema::{AppId, Exposure, FilePath, KeyKind, KeyPath, Label, Page, Schema};
    use std::ffi::OsString;

    fn sample() -> Schema {
        Schema {
            app: AppId("quire".to_owned()),
            file: FilePath("quire/appearance.toml".to_owned()),
            version: 1,
            foreign: Default::default(),
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
                labels: Default::default(),
                agent: Default::default(),
                unavailable: Default::default(),
                groups: Default::default(),
                places: Default::default(),
                note: None,
            }],
        }
    }

    #[test]
    fn a_never_settable_key_marked_settable_is_rejected() {
        const NEVER: &[&str] = &[
            "agent.budget",
            "ai.model",
            "cua.enabled",
            "memory.on",
            "companion.mood",
            "session.lock_grace_s",
            "idle.dim_after_s",
            "accounts.signed_in",
            "privacy.share_usage",
            "network.proxy",
            "security.require_password",
        ];
        for path in NEVER {
            let mut schema = sample();
            schema.key[0].path = KeyPath((*path).to_owned());
            schema.key[0].agent = crate::schema::AgentSetting::Settable;
            let err = Schema::from_toml(&schema.to_toml()).expect_err(path);
            assert!(err.contains(path), "{err}");
            // The same key left hands-off is fine, and so is a settable key elsewhere.
            schema.key[0].agent = crate::schema::AgentSetting::HandsOff;
            assert_eq!(Schema::from_toml(&schema.to_toml()), Ok(schema));
        }
        let mut ok = sample();
        ok.key[0].agent = crate::schema::AgentSetting::Settable;
        assert_eq!(Schema::from_toml(&ok.to_toml()), Ok(ok));
    }

    #[test]
    fn discover_skips_a_schema_that_marks_a_never_key_settable() {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{}",
            std::process::id(),
            "never-settable"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut schema = sample();
        schema.key[0].path = KeyPath("cua.enabled".to_owned());
        schema.key[0].agent = crate::schema::AgentSetting::Settable;
        schema.write_to(&dir).unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());
        std::fs::remove_dir_all(&dir).unwrap();
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

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_user_schema_never_shadows_an_installed_one_of_the_same_program() {
        let system = scratch("shadow-system");
        let user = scratch("shadow-user");
        let mut installed = sample();
        installed.key[0].agent = crate::schema::AgentSetting::HandsOff;
        installed.write_to(&system).unwrap();
        let mut planted = sample();
        planted.key[0].agent = crate::schema::AgentSetting::Settable;
        planted.write_to(&user).unwrap();
        // The system directory is listed first, as `data_dirs` lists it.
        let found = discover(&[system.clone(), user.clone()]);
        assert_eq!(found, vec![installed]);
        // A user schema for a program nobody installed is still found.
        let mut own = sample();
        own.app = AppId("mine".to_owned());
        own.file = FilePath("mine/settings.toml".to_owned());
        own.write_to(&user).unwrap();
        assert_eq!(discover(&[system.clone(), user.clone()]).len(), 2);
        std::fs::remove_dir_all(&system).unwrap();
        std::fs::remove_dir_all(&user).unwrap();
    }

    #[test]
    fn a_schema_that_names_another_programs_file_or_name_is_not_admitted() {
        let dir = scratch("binding");
        let mut other_file = sample();
        other_file.file = FilePath("sill/settings.toml".to_owned());
        other_file.write_to(&dir).unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());

        // Named for one program, declaring another.
        std::fs::remove_file(dir.join("quire.settings.toml")).unwrap();
        std::fs::write(dir.join("sill.settings.toml"), sample().to_toml()).unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_huge_schema_file_is_not_read() {
        let dir = scratch("huge");
        let text = format!("{}\n# {}", sample().to_toml(), "x".repeat(300 * 1024));
        std::fs::write(dir.join("quire.settings.toml"), text).unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_nonexistent_directory_is_skipped_silently() {
        let dir = std::env::temp_dir().join("ds-settings-schema-test-does-not-exist");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(discover(&[dir]), Vec::new());
    }

    #[test]
    fn a_malformed_file_is_skipped() {
        let dir = std::env::temp_dir().join(format!(
            "ds-settings-schema-test-{}-{}",
            std::process::id(),
            "malformed"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("broken.settings.toml"), "not = [valid").unwrap();
        assert_eq!(discover(std::slice::from_ref(&dir)), Vec::new());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn xdg_data_home_comes_after_xdg_data_dirs_so_it_never_shadows_them() {
        let dirs = data_dirs_from(
            Some(OsString::from("/xdg-data-home")),
            Some(OsString::from("/a:/b")),
            Some(OsString::from("/home/ada")),
        );
        assert_eq!(
            dirs,
            vec![
                std::path::PathBuf::from("/a/quire/settings"),
                std::path::PathBuf::from("/b/quire/settings"),
                std::path::PathBuf::from("/xdg-data-home/quire/settings"),
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
            dirs.last(),
            Some(&std::path::PathBuf::from(
                "/home/ada/.local/share/quire/settings"
            ))
        );
    }

    #[test]
    fn empty_or_relative_values_are_ignored_not_used_as_given() {
        let dirs = data_dirs_from(
            Some(OsString::from("")),
            Some(OsString::from("")),
            Some(OsString::from("/home/ada")),
        );
        assert_eq!(
            dirs,
            vec![
                std::path::PathBuf::from("/usr/local/share/quire/settings"),
                std::path::PathBuf::from("/usr/share/quire/settings"),
                std::path::PathBuf::from("/home/ada/.local/share/quire/settings"),
            ]
        );
        let relative = data_dirs_from(
            Some(OsString::from("rel")),
            Some(OsString::from("rel:/a")),
            None,
        );
        assert_eq!(
            relative,
            vec![std::path::PathBuf::from("/a/quire/settings")]
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

#[cfg(test)]
mod foreign_tests {
    use super::{ForeignTables, Schema};

    #[test]
    fn foreign_tables_round_trip_and_are_left_out_when_empty() {
        let text = "app = \"inferd\"\nfile = \"inferd/inferd.toml\"\nversion = 1\nforeign = [\"engines\", \"probe\"]\nkey = []\n";
        let schema = Schema::from_toml(text).expect("parses");
        assert_eq!(
            schema.foreign,
            ForeignTables(vec!["engines".to_owned(), "probe".to_owned()])
        );
        assert!(
            schema
                .to_toml()
                .contains("foreign = [\"engines\", \"probe\"]")
        );
        let plain = Schema {
            foreign: ForeignTables::default(),
            ..schema
        };
        assert!(!plain.to_toml().contains("foreign"));
    }

    #[test]
    fn a_path_is_held_by_its_first_segment_only() {
        let foreign = ForeignTables(vec!["engines".to_owned()]);
        let cases = [
            ("engines", true),
            ("engines.local.cmd", true),
            ("engine.local", false),
            ("probe.engines", false),
        ];
        for (path, held) in cases {
            assert_eq!(foreign.holds(path), held, "{path}");
        }
    }
}
