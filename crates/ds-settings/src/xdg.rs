//! The XDG Base Directory rules, written once: every place this crate turns `XDG_*` and `HOME`
//! into a directory goes through here (the config root, the schema directories, the icon assets).
//!
//! The specification's rules, as functions of values handed in (never read from the process
//! environment here, so a table test can pass any value):
//!
//! - a variable that is empty is the same as one that is unset;
//! - a path that is not absolute is invalid and ignored, as if the variable were unset;
//! - `$HOME` is held to the same two rules (an empty or relative one is absent), so a
//!   fallback under it is never a relative path under the working directory;
//! - an `$XDG_*_HOME` that is unset or ignored falls back to a directory under `$HOME`, and to
//!   nothing when `$HOME` is absent too;
//! - `$XDG_DATA_DIRS` is a `:`-separated list: unset or empty means the default list, and an
//!   empty or relative entry is dropped from it.

use std::ffi::OsStr;
use std::path::PathBuf;

/// What `$XDG_DATA_DIRS` is when it is unset or empty.
pub(crate) const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// The default of `$XDG_CONFIG_HOME`, under `$HOME`.
pub(crate) const CONFIG_HOME_UNDER_HOME: &str = ".config";

/// The default of `$XDG_DATA_HOME`, under `$HOME`.
pub(crate) const DATA_HOME_UNDER_HOME: &str = ".local/share";

/// `value` as an absolute path: `None` when it is unset, empty or relative.
fn absolute(value: Option<&OsStr>) -> Option<PathBuf> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

/// A `$XDG_*_HOME` directory: `xdg` when it is a usable absolute path, else `$HOME/<under_home>`
/// when `home` is one, else `None`.
pub(crate) fn base_dir(
    xdg: Option<&OsStr>,
    home: Option<&OsStr>,
    under_home: &str,
) -> Option<PathBuf> {
    absolute(xdg).or_else(|| absolute(home).map(|home| home.join(under_home)))
}

/// The directories of a `:`-separated search list such as `$XDG_DATA_DIRS`: `value` when it is
/// set and non-empty, else [`DEFAULT_DATA_DIRS`], with every empty or relative entry dropped.
pub(crate) fn search_dirs(value: Option<&OsStr>) -> Vec<PathBuf> {
    let list = value
        .filter(|value| !value.is_empty())
        .unwrap_or(OsStr::new(DEFAULT_DATA_DIRS));
    std::env::split_paths(list)
        .filter(|dir| dir.is_absolute())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_HOME_UNDER_HOME, DATA_HOME_UNDER_HOME, base_dir, search_dirs};
    use std::ffi::OsStr;
    use std::path::PathBuf;

    struct BaseCase {
        name: &'static str,
        xdg: Option<&'static str>,
        home: Option<&'static str>,
        expect: Option<&'static str>,
    }

    const fn case(
        name: &'static str,
        xdg: Option<&'static str>,
        home: Option<&'static str>,
        expect: Option<&'static str>,
    ) -> BaseCase {
        BaseCase {
            name,
            xdg,
            home,
            expect,
        }
    }

    /// Every combination of unset, empty, relative and absolute for the variable and for `$HOME`.
    const BASE_CASES: &[BaseCase] = &[
        case(
            "xdg absolute wins",
            Some("/xdg"),
            Some("/home/ada"),
            Some("/xdg"),
        ),
        case("xdg absolute, home unset", Some("/xdg"), None, Some("/xdg")),
        case(
            "xdg absolute, home empty",
            Some("/xdg"),
            Some(""),
            Some("/xdg"),
        ),
        case(
            "xdg absolute, home relative",
            Some("/xdg"),
            Some("h"),
            Some("/xdg"),
        ),
        case("xdg unset", None, Some("/home/ada"), Some("/home/ada/x")),
        case(
            "xdg empty",
            Some(""),
            Some("/home/ada"),
            Some("/home/ada/x"),
        ),
        case(
            "xdg relative",
            Some("rel/dir"),
            Some("/home/ada"),
            Some("/home/ada/x"),
        ),
        case("xdg dot", Some("."), Some("/home/ada"), Some("/home/ada/x")),
        case("xdg unset, home unset", None, None, None),
        case("xdg unset, home empty", None, Some(""), None),
        case("xdg unset, home relative", None, Some("home"), None),
        case("xdg empty, home unset", Some(""), None, None),
        case("xdg empty, home empty", Some(""), Some(""), None),
        case("xdg empty, home relative", Some(""), Some("home"), None),
        case("xdg relative, home unset", Some("rel"), None, None),
        case("xdg relative, home empty", Some("rel"), Some(""), None),
        case(
            "xdg relative, home relative",
            Some("rel"),
            Some("home"),
            None,
        ),
    ];

    #[test]
    fn a_home_directory_follows_the_specification() {
        for case in BASE_CASES {
            // A home fallback lands under `x`; the real defaults are checked below.
            let got = base_dir(case.xdg.map(OsStr::new), case.home.map(OsStr::new), "x");
            assert_eq!(got, case.expect.map(PathBuf::from), "{}", case.name);
        }
    }

    #[test]
    fn the_defaults_sit_under_home() {
        let home = Some(OsStr::new("/home/ada"));
        assert_eq!(
            base_dir(None, home, CONFIG_HOME_UNDER_HOME),
            Some(PathBuf::from("/home/ada/.config"))
        );
        assert_eq!(
            base_dir(Some(OsStr::new("")), home, DATA_HOME_UNDER_HOME),
            Some(PathBuf::from("/home/ada/.local/share"))
        );
    }

    #[test]
    fn a_search_list_follows_the_specification() {
        const CASES: &[(&str, Option<&str>, &[&str])] = &[
            (
                "unset is the default",
                None,
                &["/usr/local/share", "/usr/share"],
            ),
            (
                "empty is the default",
                Some(""),
                &["/usr/local/share", "/usr/share"],
            ),
            (
                "absolute entries kept in order",
                Some("/b:/a"),
                &["/b", "/a"],
            ),
            ("relative entries dropped", Some("rel:/a:./b"), &["/a"]),
            ("empty entries dropped", Some(":/a::/b:"), &["/a", "/b"]),
            ("only invalid entries leaves none", Some("rel:."), &[]),
        ];
        for (name, value, want) in CASES {
            let want: Vec<PathBuf> = want.iter().map(PathBuf::from).collect();
            assert_eq!(search_dirs(value.map(OsStr::new)), want, "{name}");
        }
    }
}
