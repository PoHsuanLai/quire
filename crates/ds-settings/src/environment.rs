//! Everything a surface resolves its look from, as one live signal: the settings file and the
//! desktop's preferences, both watched.

use crate::dirs::{self, AppName};
use crate::file;
use crate::portal::{self, SystemPrefsWatch};
use crate::settings::AppearanceFile;
use crate::watch::{self, AppearanceWatch};
use dioxus::prelude::*;
use ds::SystemPrefs;

/// The inputs to [`ds::resolve`], as they are now.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    /// `appearance.toml`.
    pub settings: AppearanceFile,
    /// The portal's answer.
    pub system: SystemPrefs,
}

/// mailo's legacy settings file name, inside its own config directory.
const LEGACY_FILE_NAME: &str = "appearance.json";

/// `app`'s settings, importing mailo's `appearance.json` once when `app` is not mailo itself
/// (there is nothing to import mailo's settings *from*), else the defaults for a program with no
/// config directory at all.
fn load_initial(app: AppName) -> AppearanceFile {
    let Some(dir) = dirs::config_dir(app) else {
        return AppearanceFile::default();
    };
    if app == AppName::MAILO {
        return file::load(&dir);
    }
    match dirs::config_dir(AppName::MAILO) {
        Some(mailo_dir) => {
            file::load_or_import(&dir, &mailo_dir.join(LEGACY_FILE_NAME)).unwrap_or_default()
        }
        None => file::load(&dir),
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
