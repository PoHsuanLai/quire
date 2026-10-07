//! Where a program's settings live: [`ConfigRoot`] says which directory tree, [`AppName`] which
//! program's directory inside it.
//!
//! Config, not data: a cosmetic choice is never a write to the file that holds someone's mail.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::xdg;

/// The directory name a program's files live under: `quire`, `sill`, `mailo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppName(pub &'static str);

impl AppName {
    /// The design system's own settings (`appearance.toml`).
    pub const QUIRE: AppName = AppName("quire");
    /// mailo.
    pub const MAILO: AppName = AppName("mailo");
}

/// The tree every program's config directory sits in.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigRoot {
    /// The person's own: `$XDG_CONFIG_HOME`, else `$HOME/.config`, by the XDG rules in
    /// `crate::xdg` (an empty or relative value is ignored). The one place this crate reads
    /// the environment for the config directory.
    Xdg,
    /// A directory standing in for `$XDG_CONFIG_HOME`: a test's scratch directory, a dev
    /// script's private home.
    Scratch(PathBuf),
}

impl ConfigRoot {
    /// `app`'s config directory in this tree, or `None` when [`ConfigRoot::Xdg`] has neither
    /// variable to start from.
    pub fn dir(&self, app: AppName) -> Option<PathBuf> {
        match self {
            ConfigRoot::Xdg => xdg_dir(
                app,
                std::env::var_os("XDG_CONFIG_HOME"),
                std::env::var_os("HOME"),
            ),
            ConfigRoot::Scratch(base) => Some(base.join(app.0)),
        }
    }
}

impl ConfigRoot {
    /// `app`'s state directory in this tree: what changes as the program is used and is not a
    /// choice (`$XDG_STATE_HOME`, else `$HOME/.local/state`). A scratch root keeps it beside the
    /// config directory, in `.state`.
    pub fn state_dir(&self, app: AppName) -> Option<PathBuf> {
        match self {
            ConfigRoot::Xdg => xdg::base_dir(
                std::env::var_os("XDG_STATE_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
                xdg::STATE_HOME_UNDER_HOME,
            )
            .map(|base| base.join(app.0)),
            ConfigRoot::Scratch(base) => Some(base.join(".state").join(app.0)),
        }
    }
}

fn xdg_dir(app: AppName, xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    xdg::base_dir(xdg.as_deref(), home.as_deref(), xdg::CONFIG_HOME_UNDER_HOME)
        .map(|base| base.join(app.0))
}

#[cfg(test)]
mod tests {
    use super::{AppName, ConfigRoot, xdg_dir};
    use std::ffi::OsString;
    use std::path::PathBuf;

    struct Case {
        name: &'static str,
        xdg: Option<&'static str>,
        home: Option<&'static str>,
        expect: Option<&'static str>,
    }

    const CASES: &[Case] = &[
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
            name: "home when xdg is empty",
            xdg: Some(""),
            home: Some("/home/ada"),
            expect: Some("/home/ada/.config/quire"),
        },
        Case {
            name: "home when xdg is relative",
            xdg: Some("config"),
            home: Some("/home/ada"),
            expect: Some("/home/ada/.config/quire"),
        },
        Case {
            name: "neither",
            xdg: None,
            home: None,
            expect: None,
        },
        Case {
            name: "empty xdg and empty home is not the working directory",
            xdg: Some(""),
            home: Some(""),
            expect: None,
        },
        Case {
            name: "relative home is not a home",
            xdg: None,
            home: Some("ada"),
            expect: None,
        },
    ];

    #[test]
    fn the_config_directory_follows_xdg() {
        for case in CASES {
            assert_eq!(
                xdg_dir(
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
    fn the_app_name_is_the_last_component() {
        assert_eq!(
            xdg_dir(AppName::MAILO, Some(OsString::from("/xdg")), None),
            Some(PathBuf::from("/xdg/mailo"))
        );
    }

    #[test]
    fn a_scratch_root_stands_in_for_xdg_config_home() {
        let root = ConfigRoot::Scratch(PathBuf::from("/scratch"));
        assert_eq!(
            root.dir(AppName::QUIRE),
            Some(PathBuf::from("/scratch/quire"))
        );
    }
}
