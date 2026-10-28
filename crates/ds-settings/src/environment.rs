//! Everything a surface resolves its look from, as one live signal: the settings file and the
//! desktop's preferences, both watched.

use crate::appearance::AppearanceFile;
use crate::doc::SettingsDoc;
use crate::lenient::Loaded;
use crate::portal::{SystemPrefsSource, SystemPrefsWatch};
use crate::region::ResolvedHours;
use crate::store::Store;
use crate::watch::Watch;
use dioxus::prelude::*;
use ds_core::spawner::Spawner;
use ds_style::appearance::system::SystemPrefs;
use std::sync::Arc;

/// The inputs to [`ds::prelude::resolve`], as they are now.
///
/// A consumer wires the environment into its root like this:
///
/// ```no_run
/// use dioxus::prelude::*;
/// use ds::prelude::*; use ds_core::spawner::Spawner;
/// use ds_settings::{AppName, ConfigRoot, Store, SystemPrefsSource, use_environment};
/// use std::sync::Arc;
///
/// #[component]
/// fn Root(children: Element) -> Element {
///     let spawn = use_context::<Arc<dyn Spawner>>();
///     let store = Store::new(ConfigRoot::Xdg, AppName::MAILO);
///     let env = use_environment(store, SystemPrefsSource::Portal, spawn);
///     let now = env();
///     rsx! {
///         Ds {
///             appearance: now.settings.appearance.appearance(),
///             system: now.system,
///             custom_styles: now.settings.appearance.custom_styles,
///             material: Material::Window,
///             {children}
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    /// `appearance.toml`.
    pub settings: AppearanceFile,
    /// The portal's answer.
    pub system: SystemPrefs,
}

impl Environment {
    /// The clock to draw in `locale` (a POSIX locale name, see
    /// [`time_locale`](crate::time_locale)): `region.hours`, with Automatic settled by the locale.
    pub fn hours(&self, locale: &str) -> ResolvedHours {
        self.settings.region.hours.resolve(locale)
    }
}

/// Print what the file held that did not become a setting, once per read.
fn report(loaded: &Loaded<AppearanceFile>) {
    for line in loaded.diagnostics(AppearanceFile::FILE) {
        eprintln!("ds-settings: {line}");
    }
}

/// Load the settings `store` holds, read the desktop's preferences from `system`, and keep both
/// live, their watches running on `spawner`. Unknown and invalid keys in the file are printed to
/// stderr as they are read.
///
/// The person's own file comes first: it is published at once and watched before the desktop is
/// asked anything, so a portal that is slow, wedged or absent never delays the appearance the
/// person chose or hides an edit to it. The desktop's preferences fold in when the portal has
/// answered (or, if it does not answer in time, never, and the defaults stand).
pub fn use_environment(
    store: Store,
    system: SystemPrefsSource,
    spawner: Arc<dyn Spawner>,
) -> ReadSignal<Environment> {
    let mut env = use_signal(Environment::default);

    use_future(move || {
        let (store, system, spawner) = (store.clone(), system.clone(), spawner.clone());
        async move {
            let loaded = store.load::<AppearanceFile>();
            report(&loaded);
            env.with_mut(|e| e.settings = loaded.value);
            let files = store.watch::<AppearanceFile>(&*spawner);
            spawn(follow_files(env, files));
            let prefs = SystemPrefsWatch::start(&system, &*spawner).await;
            env.with_mut(|e| e.system = prefs.current());
            follow_prefs(env, prefs).await;
        }
    });

    ReadSignal::new(env)
}

/// Apply every settled change of the settings file to `env` until the watch stops.
async fn follow_files(mut env: Signal<Environment>, mut files: Watch<AppearanceFile>) {
    while let Some(loaded) = files.changed().await {
        report(&loaded);
        env.with_mut(|e| e.settings = loaded.value);
    }
}

/// Apply every change of the desktop's preferences to `env` until the watch stops.
async fn follow_prefs(mut env: Signal<Environment>, mut prefs: SystemPrefsWatch) {
    while let Some(system) = prefs.changed().await {
        env.with_mut(|e| e.system = system);
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;

    #[test]
    fn the_clock_follows_the_hours_key_and_then_the_locale() {
        use crate::region::{HourCycle, ResolvedHours};
        let mut env = Environment::default();
        assert_eq!(env.hours("en_US.UTF-8"), ResolvedHours::H12);
        assert_eq!(env.hours("de_DE"), ResolvedHours::H24);
        env.settings.region.hours = HourCycle::H24;
        assert_eq!(env.hours("en_US.UTF-8"), ResolvedHours::H24);
    }
}
