//! Everything a surface resolves its look from, as one live signal: the settings file and the
//! desktop's preferences, both watched.

use crate::appearance::AppearanceFile;
use crate::doc::SettingsDoc;
use crate::lenient::Loaded;
use crate::portal::{SystemPrefsSource, SystemPrefsWatch};
use crate::store::Store;
use crate::units::Percent;
use crate::watch::Watch;
use dioxus::prelude::*;
use ds::SystemPrefs;
use ds_core::spawner::Spawner;
use std::sync::Arc;

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
    /// use ds::{Ds, Material, Spawner};
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
    ///             tint_alpha: Some(now.tint_alpha()),
    ///             typeface: Some(now.settings.appearance.typeface()),
    ///             material: Material::Window,
    ///             {children}
    ///         }
    ///     }
    /// }
    /// ```
    pub fn tint_alpha(&self) -> ds::Alpha {
        ds::Alpha(u16::from(self.settings.appearance.material_tint_alpha.0.min(100)) * 10)
    }

    /// The six `appearance.material_*` keys as the root's `stack` (`Ds { stack }`): highlight
    /// and hairline alphas per scheme, shadow strength and vibrancy, each a percent in
    /// thousandths like [`Environment::tint_alpha`].
    pub fn material_stack(&self) -> ds::MaterialStack {
        let a = &self.settings.appearance;
        let alpha = |percent: Percent| ds::Alpha(u16::from(percent.0.min(100)) * 10);
        ds::MaterialStack {
            highlight_light: alpha(a.material_highlight_light),
            highlight_dark: alpha(a.material_highlight_dark),
            hairline_light: alpha(a.material_hairline_light),
            hairline_dark: alpha(a.material_hairline_dark),
            shadow_strength: alpha(a.material_shadow_strength),
            vibrancy: alpha(a.material_vibrancy),
        }
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
            let prefs = SystemPrefsWatch::start(&system, &*spawner).await;
            env.set(Environment {
                settings: loaded.value,
                system: prefs.current(),
            });
            let files = store.watch::<AppearanceFile>(&*spawner);
            spawn(follow_files(env, files));
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
    use crate::units::Percent;

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

    #[test]
    fn the_default_material_keys_are_the_stacks_defaults() {
        let env = Environment::default();
        assert_eq!(env.material_stack(), ds::MaterialStack::default());
    }
}
