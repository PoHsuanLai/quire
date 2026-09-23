//! Where settings, state and caches live, per program: `$XDG_CONFIG_HOME/<app>` and friends.
//!
//! Moved from mailo (`mail-app/src/appearance.rs`), parameterised by [`AppName`] instead of
//! hard-coding `mailo`. Config, not data: a cosmetic choice is never a write to the file that
//! holds someone's mail.

use std::ffi::OsString;
use std::path::PathBuf;

/// The directory name a program's files live under: `quire`, `sill`, `mailo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppName(pub &'static str);

impl AppName {
    /// The design system's own settings (`appearance.toml`).
    pub const QUIRE: AppName = AppName("quire");
    /// mailo, whose `appearance.json` is imported once.
    pub const MAILO: AppName = AppName("mailo");
}

/// `$XDG_CONFIG_HOME/<app>`, else `$HOME/.config/<app>`, else None.
pub fn config_dir(app: AppName) -> Option<PathBuf> {
    config_dir_from(
        app,
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

fn config_dir_from(app: AppName, xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = xdg
        .map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join(app.0))
}

/// `$XDG_STATE_HOME/<app>`, else `$HOME/.local/state/<app>`, else None.
///
/// State, not a preference: lists of what was opened, which expire.
pub fn state_dir(app: AppName) -> Option<PathBuf> {
    state_dir_from(
        app,
        std::env::var_os("XDG_STATE_HOME"),
        std::env::var_os("HOME"),
    )
}

fn state_dir_from(app: AppName, xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = xdg
        .map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".local/state")))?;
    Some(base.join(app.0))
}

/// `$XDG_CACHE_HOME/<app>`, else `$HOME/.cache/<app>`, else None.
///
/// A missing cache is a fallback drawn, not a lost setting.
pub fn cache_dir(app: AppName) -> Option<PathBuf> {
    cache_dir_from(
        app,
        std::env::var_os("XDG_CACHE_HOME"),
        std::env::var_os("HOME"),
    )
}

fn cache_dir_from(app: AppName, xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = xdg
        .map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".cache")))?;
    Some(base.join(app.0))
}

#[cfg(test)]
mod tests {
    use super::{AppName, cache_dir_from, config_dir_from, state_dir_from};
    use std::ffi::OsString;
    use std::path::PathBuf;

    struct Case {
        name: &'static str,
        xdg: Option<&'static str>,
        home: Option<&'static str>,
        expect: Option<&'static str>,
    }

    type Resolver = fn(AppName, Option<OsString>, Option<OsString>) -> Option<PathBuf>;

    fn run(resolver: Resolver, cases: &[Case]) {
        for case in cases {
            assert_eq!(
                resolver(
                    AppName::QUIRE,
                    case.xdg.map(OsString::from),
                    case.home.map(OsString::from)
                ),
                case.expect.map(PathBuf::from),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn the_config_directory_follows_xdg() {
        run(
            config_dir_from,
            &[
                Case {
                    name: "xdg wins",
                    xdg: Some("/xdg"),
                    home: Some("/home/ada"),
                    expect: Some("/xdg/quire"),
                },
                Case {
                    name: "home when xdg is unset",
                    xdg: None,
                    home: Some("/home/ada"),
                    expect: Some("/home/ada/.config/quire"),
                },
                Case {
                    name: "neither",
                    xdg: None,
                    home: None,
                    expect: None,
                },
            ],
        );
    }

    #[test]
    fn the_state_directory_follows_xdg() {
        run(
            state_dir_from,
            &[
                Case {
                    name: "xdg wins",
                    xdg: Some("/xdg"),
                    home: Some("/home/ada"),
                    expect: Some("/xdg/quire"),
                },
                Case {
                    name: "home when xdg is unset",
                    xdg: None,
                    home: Some("/home/ada"),
                    expect: Some("/home/ada/.local/state/quire"),
                },
                Case {
                    name: "neither",
                    xdg: None,
                    home: None,
                    expect: None,
                },
            ],
        );
    }

    #[test]
    fn the_cache_directory_follows_xdg() {
        run(
            cache_dir_from,
            &[
                Case {
                    name: "xdg wins",
                    xdg: Some("/xdg"),
                    home: Some("/home/ada"),
                    expect: Some("/xdg/quire"),
                },
                Case {
                    name: "home when xdg is unset",
                    xdg: None,
                    home: Some("/home/ada"),
                    expect: Some("/home/ada/.cache/quire"),
                },
                Case {
                    name: "neither",
                    xdg: None,
                    home: None,
                    expect: None,
                },
            ],
        );
    }

    #[test]
    fn the_app_name_is_the_last_component() {
        assert_eq!(
            config_dir_from(AppName::MAILO, Some(OsString::from("/xdg")), None),
            Some(PathBuf::from("/xdg/mailo"))
        );
    }
}
