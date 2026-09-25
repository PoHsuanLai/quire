//! The apps directory's lookup order (see the module doc), as data and a probe.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The directory under a data directory that holds the app icons.
const SUBDIR: &str = "quire/icons/apps";

/// `$XDG_DATA_DIRS` when it is unset or empty (the XDG base directory specification).
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// The repository's own set, for a path-dependency consumer in development.
const DEV_ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icons/apps");

/// Which step of the lookup a directory came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetsOrigin {
    /// `$QUIRE_ICON_ASSETS`.
    Override,
    /// `$XDG_DATA_HOME/quire/icons/apps`.
    DataHome,
    /// One `$XDG_DATA_DIRS` entry's `quire/icons/apps`.
    DataDir,
    /// The repository's `assets/icons/apps`, from `CARGO_MANIFEST_DIR` at compile time:
    /// development only; an installed program never has it.
    DevAssets,
}

/// One place the icons may be, and which step named it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetsDir {
    /// The apps directory: `<it>/<app>/<px>.png`.
    pub path: PathBuf,
    /// The step of the lookup it came from.
    pub origin: AssetsOrigin,
}

/// Whether a path is on disk as the lookup needs it (a directory, or a file).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence {
    /// It is there.
    Present,
    /// It is not.
    Absent,
}

/// The variables the lookup reads, captured once so the order is a pure function of them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssetsEnv {
    /// `$QUIRE_ICON_ASSETS`.
    pub quire_icon_assets: Option<OsString>,
    /// `$XDG_DATA_HOME`.
    pub xdg_data_home: Option<OsString>,
    /// `$HOME`, for the data home's default.
    pub home: Option<OsString>,
    /// `$XDG_DATA_DIRS`.
    pub xdg_data_dirs: Option<OsString>,
}

impl AssetsEnv {
    /// This process's environment.
    pub fn current() -> AssetsEnv {
        AssetsEnv {
            quire_icon_assets: std::env::var_os("QUIRE_ICON_ASSETS"),
            xdg_data_home: std::env::var_os("XDG_DATA_HOME"),
            home: std::env::var_os("HOME"),
            xdg_data_dirs: std::env::var_os("XDG_DATA_DIRS"),
        }
    }
}

/// A variable's value, with an empty one read as unset (the XDG specification's rule).
fn set(value: &Option<OsString>) -> Option<&OsString> {
    value.as_ref().filter(|value| !value.is_empty())
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`; a relative value is ignored, as the XDG
/// specification says.
fn data_home(env: &AssetsEnv) -> Option<PathBuf> {
    set(&env.xdg_data_home)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| set(&env.home).map(|home| PathBuf::from(home).join(".local/share")))
}

/// Every place to look, in order; none of them checked yet.
pub fn candidates(env: &AssetsEnv) -> Vec<AssetsDir> {
    let at = |origin| move |path: PathBuf| AssetsDir { path, origin };
    let data_dirs = set(&env.xdg_data_dirs)
        .cloned()
        .unwrap_or_else(|| OsString::from(DEFAULT_DATA_DIRS));
    let shared = std::env::split_paths(&data_dirs)
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join(SUBDIR));
    set(&env.quire_icon_assets)
        .map(PathBuf::from)
        .map(at(AssetsOrigin::Override))
        .into_iter()
        .chain(
            data_home(env)
                .map(|home| home.join(SUBDIR))
                .map(at(AssetsOrigin::DataHome)),
        )
        .chain(shared.map(at(AssetsOrigin::DataDir)))
        .chain([at(AssetsOrigin::DevAssets)(PathBuf::from(DEV_ASSETS))])
        .collect()
}

/// The first candidate `probe` finds.
pub fn find_apps_dir(env: &AssetsEnv, probe: impl Fn(&Path) -> Presence) -> Option<AssetsDir> {
    candidates(env)
        .into_iter()
        .find(|dir| probe(&dir.path) == Presence::Present)
}

/// Whether `path` is a directory on disk.
pub(super) fn a_directory(path: &Path) -> Presence {
    match path.is_dir() {
        true => Presence::Present,
        false => Presence::Absent,
    }
}

/// Whether `path` is a file on disk.
pub(super) fn a_file(path: &Path) -> Presence {
    match path.is_file() {
        true => Presence::Present,
        false => Presence::Absent,
    }
}

/// The apps directory for this process: the first of the lookup order that is a directory.
pub fn apps_dir() -> Option<PathBuf> {
    find_apps_dir(&AssetsEnv::current(), a_directory).map(|dir| dir.path)
}

/// Where `icons install` copies the set: `$XDG_DATA_HOME/quire/icons/apps` (step 2).
pub fn install_dir(env: &AssetsEnv) -> Option<PathBuf> {
    data_home(env).map(|home| home.join(SUBDIR))
}

#[cfg(test)]
mod tests {
    use super::{
        AssetsEnv, AssetsOrigin, DEV_ASSETS, Presence, candidates, find_apps_dir, install_dir,
    };
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    fn env(assets: Option<&str>, home: Option<&str>, dirs: Option<&str>) -> AssetsEnv {
        AssetsEnv {
            quire_icon_assets: assets.map(OsString::from),
            xdg_data_home: home.map(OsString::from),
            home: Some(OsString::from("/home/ada")),
            xdg_data_dirs: dirs.map(OsString::from),
        }
    }

    /// A listed candidate: its step and its path.
    type Step<'a> = (AssetsOrigin, &'a str);

    fn listed(env: &AssetsEnv) -> Vec<(AssetsOrigin, String)> {
        candidates(env)
            .into_iter()
            .map(|dir| (dir.origin, dir.path.display().to_string()))
            .collect()
    }

    #[test]
    fn the_order_is_override_data_home_data_dirs_then_the_repository() {
        use AssetsOrigin::{DataDir, DataHome, DevAssets, Override};
        let dev = DEV_ASSETS.to_owned();
        let cases: &[(&str, AssetsEnv, Vec<Step>)] = &[
            (
                "everything set",
                env(Some("/opt/icons"), Some("/data"), Some("/a:/b")),
                vec![
                    (Override, "/opt/icons"),
                    (DataHome, "/data/quire/icons/apps"),
                    (DataDir, "/a/quire/icons/apps"),
                    (DataDir, "/b/quire/icons/apps"),
                    (DevAssets, &dev),
                ],
            ),
            (
                "nothing set: the XDG defaults",
                env(None, None, None),
                vec![
                    (DataHome, "/home/ada/.local/share/quire/icons/apps"),
                    (DataDir, "/usr/local/share/quire/icons/apps"),
                    (DataDir, "/usr/share/quire/icons/apps"),
                    (DevAssets, &dev),
                ],
            ),
            (
                "empty values read as unset; relative entries are skipped",
                env(Some(""), Some("rel"), Some(":/c:rel")),
                vec![
                    (DataHome, "/home/ada/.local/share/quire/icons/apps"),
                    (DataDir, "/c/quire/icons/apps"),
                    (DevAssets, &dev),
                ],
            ),
        ];
        for (name, env, want) in cases {
            let want: Vec<(AssetsOrigin, String)> = want
                .iter()
                .map(|(origin, path)| (*origin, (*path).to_owned()))
                .collect();
            assert_eq!(listed(env), want, "{name}");
        }
    }

    #[test]
    fn the_first_present_directory_wins() {
        let everything = env(Some("/opt/icons"), Some("/data"), Some("/a:/b"));
        let cases: &[(&str, &[&str], Option<Step>)] = &[
            (
                "the override",
                &["/opt/icons", "/data/quire/icons/apps"],
                Some((AssetsOrigin::Override, "/opt/icons")),
            ),
            (
                "an installed set over a system one",
                &["/data/quire/icons/apps", "/b/quire/icons/apps"],
                Some((AssetsOrigin::DataHome, "/data/quire/icons/apps")),
            ),
            (
                "the second system dir",
                &["/b/quire/icons/apps"],
                Some((AssetsOrigin::DataDir, "/b/quire/icons/apps")),
            ),
            ("nothing anywhere", &[], None),
        ];
        for (name, present, want) in cases {
            let probe = |path: &Path| match present.iter().any(|p| Path::new(p) == path) {
                true => Presence::Present,
                false => Presence::Absent,
            };
            let got = find_apps_dir(&everything, probe).map(|dir| (dir.origin, dir.path));
            let want = want.map(|(origin, path)| (origin, PathBuf::from(path)));
            assert_eq!(got, want, "{name}");
        }
    }

    #[test]
    fn the_development_fallback_is_this_repositorys_set() {
        let dev = candidates(&AssetsEnv::default())
            .pop()
            .expect("the last candidate");
        assert_eq!(dev.origin, AssetsOrigin::DevAssets);
        assert!(
            dev.path.join("mail/48.png").is_file(),
            "{}",
            dev.path.display()
        );
        assert_eq!(
            install_dir(&env(None, Some("/data"), None)),
            Some(PathBuf::from("/data/quire/icons/apps"))
        );
    }
}
