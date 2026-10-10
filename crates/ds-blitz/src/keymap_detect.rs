//! The keymap source a launched app gets when it names none: the platform from the environment
//! (chordkit's `detect`), and for it the sources chordkit has: the KDE file, a GNOME dconf dump
//! when one is there, else the platform's conventions. Our desktop's own source (keycap) is
//! plugged in by the desktop's launcher, not found here. The environment is read through a
//! lookup the caller passes, so a test supplies its own.

use chordkit::{Desktop, HostOs, Platform};
use chordkit_gnome::GnomeSource;
use chordkit_kde::KdeSource;
use ds::keys::KeySource;
use std::path::PathBuf;

/// Where, under the config directory, a GNOME session's saved `dconf dump /` is looked for.
pub(crate) const GNOME_DUMP: &str = "chordkit/dconf-dump";

/// The source for the process the lookup describes.
pub(crate) fn detected(var: &impl Fn(&str) -> Option<String>) -> KeySource {
    source_for(platform_of(var), config_dir(var))
}

/// The platform the lookup describes (`CHORDKIT_PLATFORM` wins).
pub(crate) fn platform_of(var: &impl Fn(&str) -> Option<String>) -> Platform {
    chordkit::detect(HostOs::compiled(), var)
}

/// The user's config directory: `XDG_CONFIG_HOME`, else `~/.config`.
pub(crate) fn config_dir(var: &impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    var("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| PathBuf::from(home).join(".config")))
}

/// The source `platform` reads its shortcut settings from, under `config`.
pub(crate) fn source_for(platform: Platform, config: Option<PathBuf>) -> KeySource {
    match (platform, config) {
        (
            Platform::Linux {
                desktop: Desktop::Kde,
            },
            Some(dir),
        ) => KeySource::new(platform, Box::new(KdeSource::new(dir.join("kdeglobals")))),
        (
            Platform::Linux {
                desktop: Desktop::Gnome,
            },
            Some(dir),
        ) => KeySource::new(platform, Box::new(GnomeSource::new(dir.join(GNOME_DUMP)))),
        (platform, _) => KeySource::conventions(platform),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{Action, Chord, StandardAction};

    type Vars = &'static [(&'static str, &'static str)];

    fn lookup(vars: Vars) -> impl Fn(&str) -> Option<String> {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn the_config_directory_is_xdg_then_home() {
        let cases: &[(Vars, Option<&str>)] = &[
            (
                &[("XDG_CONFIG_HOME", "/x/cfg"), ("HOME", "/h")],
                Some("/x/cfg"),
            ),
            (
                &[("XDG_CONFIG_HOME", ""), ("HOME", "/h")],
                Some("/h/.config"),
            ),
            (&[("HOME", "/h")], Some("/h/.config")),
            (&[], None),
        ];
        for (vars, want) in cases {
            let found = config_dir(&lookup(*vars));
            assert_eq!(found, want.map(PathBuf::from), "{vars:?}");
        }
    }

    #[test]
    fn the_platform_comes_from_the_environment() {
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let cases: &[(Vars, Platform)] = &[
            (
                &[("CHORDKIT_PLATFORM", "linux-kde"), ("HOME", "/nowhere")],
                kde,
            ),
            (&[("CHORDKIT_PLATFORM", "linux-ours")], ours),
            (&[("CHORDKIT_PLATFORM", "windows")], Platform::Windows),
        ];
        for (vars, want) in cases {
            assert_eq!(detected(&lookup(*vars)).platform(), *want, "{vars:?}");
        }
    }

    #[test]
    fn a_kde_file_changes_the_keymap_and_a_missing_one_leaves_the_conventions() {
        let dir = std::env::temp_dir().join(format!("ds-blitz-keymap-{}", std::process::id()));
        let written = std::fs::create_dir_all(&dir).and_then(|()| {
            std::fs::write(dir.join("kdeglobals"), "[Shortcuts]\nCopy=Ctrl+Insert\n")
        });
        assert!(written.is_ok(), "{written:?}");
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let copy = Action::Standard(StandardAction::Copy);
        let chords_in = |config: PathBuf| {
            source_for(kde, Some(config))
                .load()
                .map(|keymap| keymap.chords_of(&copy))
        };
        let changed = chords_in(dir.clone());
        let missing = chords_in(dir.join("nowhere"));
        let _ = std::fs::remove_dir_all(&dir);
        let chord = |text: &str| text.parse::<Chord>().ok();
        assert_eq!(changed.ok(), chord("Ctrl+Insert").map(|c| vec![c]));
        assert_eq!(missing.ok(), chord("Ctrl+C").map(|c| vec![c]));
    }
}
