//! Everything a surface resolves its look from, as one live signal: the settings file and the
//! desktop's preferences, both watched.

use crate::appearance_file as file;
use crate::dirs::{self, AppName};
use crate::portal::{self, SystemPrefsWatch};
use crate::settings::AppearanceFile;
use crate::watch::{self, AppearanceWatch};
use dioxus::prelude::*;
use ds::SystemPrefs;
use std::path::Path;

/// The inputs to [`ds::resolve`], as they are now.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    /// `appearance.toml`.
    pub settings: AppearanceFile,
    /// The portal's answer.
    pub system: SystemPrefs,
}

impl Environment {
    /// `appearance.material_tint_alpha` as the root's `tint_alpha` (a percent in thousandths:
    /// 80 is 800). A consumer wires the environment into its root like this:
    ///
    /// ```no_run
    /// use dioxus::prelude::*;
    /// use ds::{Ds, Material};
    /// use ds_settings::{AppName, use_environment};
    ///
    /// #[component]
    /// fn Root(children: Element) -> Element {
    ///     let env = use_environment(AppName::MAILO);
    ///     let now = env();
    ///     rsx! {
    ///         Ds {
    ///             appearance: now.settings.appearance.appearance(),
    ///             system: now.system,
    ///             tint_alpha: Some(now.tint_alpha()),
    ///             material: Material::Window,
    ///             {children}
    ///         }
    ///     }
    /// }
    /// ```
    pub fn tint_alpha(&self) -> ds::Alpha {
        ds::Alpha(u16::from(self.settings.appearance.material_tint_alpha.0.min(100)) * 10)
    }
}

/// mailo's legacy settings file name, inside its own config directory.
const LEGACY_FILE_NAME: &str = "appearance.json";

/// `app`'s settings, importing mailo's legacy `appearance.json` once for every app, mailo
/// included, the first time the app's own `appearance.toml` does not exist; else the defaults
/// for a program with no config directory at all. There is one legacy file, mailo's, so every app
/// imports the same one; for mailo it sits beside its own `appearance.toml` (FINDINGS "mailo
/// gaps": mailo used to be skipped, so `use_environment(AppName::MAILO)` never imported it).
fn load_initial(app: AppName) -> AppearanceFile {
    initial_from(
        dirs::config_dir(app).as_deref(),
        dirs::config_dir(AppName::MAILO).as_deref(),
    )
}

/// [`load_initial`] over explicit directories: `dir` is the app's config directory, `mailo_dir`
/// mailo's (the same directory when the app is mailo).
fn initial_from(dir: Option<&Path>, mailo_dir: Option<&Path>) -> AppearanceFile {
    let Some(dir) = dir else {
        return AppearanceFile::default();
    };
    match mailo_dir {
        Some(mailo_dir) => {
            file::load_or_import(dir, &mailo_dir.join(LEGACY_FILE_NAME)).unwrap_or_default()
        }
        None => file::load(dir),
    }
}

/// Load `app`'s settings (importing mailo's JSON once), read the portal, and keep both live.
pub fn use_environment(app: AppName) -> ReadSignal<Environment> {
    let mut env = use_signal(Environment::default);

    use_future(move || async move {
        env.set(Environment {
            settings: load_initial(app),
            system: portal::read_system_prefs().await.unwrap_or_default(),
        });

        let file_watch = dirs::config_dir(app).and_then(|dir| watch::watch(&dir).ok());
        let portal_watch = SystemPrefsWatch::start().await.ok();

        match (file_watch, portal_watch) {
            (Some(files), Some(portal)) => watch_both(env, files, portal).await,
            (Some(files), None) => watch_files(env, files).await,
            (None, Some(portal)) => watch_portal(env, portal).await,
            (None, None) => {}
        }
    });

    ReadSignal::new(env)
}

/// Apply both watches to `env` until either stops.
async fn watch_both(
    mut env: Signal<Environment>,
    mut files: AppearanceWatch,
    mut portal: SystemPrefsWatch,
) {
    loop {
        tokio::select! {
            settings = files.changed() => {
                let Some(settings) = settings else { return; };
                env.with_mut(|e| e.settings = settings);
            }
            system = portal.changed() => {
                let Some(system) = system else { return; };
                env.with_mut(|e| e.system = system);
            }
        }
    }
}

/// Apply the file watch alone (the portal could not be reached).
async fn watch_files(mut env: Signal<Environment>, mut files: AppearanceWatch) {
    while let Some(settings) = files.changed().await {
        env.with_mut(|e| e.settings = settings);
    }
}

/// Apply the portal watch alone (the config directory could not be watched).
async fn watch_portal(mut env: Signal<Environment>, mut portal: SystemPrefsWatch) {
    while let Some(system) = portal.changed().await {
        env.with_mut(|e| e.system = system);
    }
}

#[cfg(test)]
mod tests {
    use super::{Environment, LEGACY_FILE_NAME, initial_from};
    use crate::test_dir::TempDir;
    use crate::units::Percent;
    use ds::Theme;

    /// mailo's own first run imports its own `appearance.json`, which sits in the directory its
    /// `appearance.toml` will be written to; a shell app imports the same file from mailo's
    /// directory. Both leave the JSON in place and write their TOML.
    #[test]
    fn every_app_mailo_included_imports_mailos_json_once() {
        let root = TempDir::new();
        let mailo = root.path().join("mailo");
        let shell = root.path().join("quire");
        std::fs::create_dir_all(&mailo).unwrap_or_else(|e| panic!("{e}"));
        let json = r#"{"theme":"dark","motion":"calm","marks":"letters"}"#;
        std::fs::write(mailo.join(LEGACY_FILE_NAME), json).unwrap_or_else(|e| panic!("{e}"));
        for (name, dir) in [("mailo", &mailo), ("a shell app", &shell)] {
            let got = initial_from(Some(dir), Some(&mailo));
            assert_eq!(got.appearance.theme, Theme::Dark, "{name}");
            assert!(dir.join("appearance.toml").exists(), "{name} wrote no TOML");
        }
        assert_eq!(
            std::fs::read_to_string(mailo.join(LEGACY_FILE_NAME)).unwrap_or_else(|e| panic!("{e}")),
            json,
            "the JSON is left as it was"
        );
        assert_eq!(initial_from(None, Some(&mailo)), Default::default());
    }

    #[test]
    fn the_tint_key_becomes_the_roots_thousandths() {
        const CASES: &[(u8, u16)] = &[(80, 800), (64, 640), (100, 1000), (0, 0)];
        for &(percent, want) in CASES {
            let mut env = Environment::default();
            env.settings.appearance.material_tint_alpha = Percent(percent);
            assert_eq!(env.tint_alpha(), ds::Alpha(want), "{percent}%");
        }
        assert_eq!(
            Environment::default().tint_alpha(),
            ds::Alpha(800),
            "the key's default is the root's default"
        );
    }
}
