//! The desktop's preferences from the settings portal:
//! `org.freedesktop.portal.Settings.ReadAll(["org.freedesktop.appearance"])` and
//! `SettingChanged`, mapped to [`ds::prelude::SystemPrefs`]. The mappings are pure tables, tested
//! against a fake `ReadAll`/`SettingChanged` payload with no bus involved; only
//! [`SystemPrefsSource::Portal`] touches `zbus`, and only on Linux with feature `quire-desktop`
//! (`portal/bus.rs`) — a desktop without the portal, or a build for a platform or feature set that
//! has none (`portal/absent.rs`), simply answers [`ds::prelude::SystemPrefs::default`].

use crate::latest::{self, Receiver};
use ds_core::spawner::Spawner;
use ds_style::appearance::system::Contrast;
use ds_style::appearance::system::ReducedMotion;
use ds_style::appearance::system::SystemPrefs;
use ds_style::appearance::theme::Scheme;

// The half that reaches `zbus`, or its stand-in with the same two entry points.
#[cfg_attr(
    all(target_os = "linux", feature = "quire-desktop"),
    path = "portal/bus.rs"
)]
#[cfg_attr(
    not(all(target_os = "linux", feature = "quire-desktop")),
    path = "portal/absent.rs"
)]
mod bus;

/// `color-scheme`: 0 no preference, 1 prefer dark, 2 prefer light. No preference is light.
pub fn scheme_from_portal(value: u32) -> Scheme {
    match value {
        1 => Scheme::Dark,
        _ => Scheme::Light,
    }
}

/// `reduced-motion`: 0 no preference, 1 reduce.
pub fn reduced_motion_from_portal(value: u32) -> ReducedMotion {
    match value {
        1 => ReducedMotion::Reduce,
        _ => ReducedMotion::NoPreference,
    }
}

/// `contrast`: 0 no preference, 1 higher contrast.
pub fn contrast_from_portal(value: u32) -> Contrast {
    match value {
        1 => Contrast::High,
        _ => Contrast::Normal,
    }
}

/// Where the desktop's preferences come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemPrefsSource {
    /// The settings portal on the session bus. A desktop without it, or a build without the portal client answers
    /// the defaults and never changes.
    Portal,
    /// These preferences, forever: a test, a headless render or a pinned appearance.
    Fixed(SystemPrefs),
}

impl SystemPrefsSource {
    /// Every appearance preference, once. Never fails: a missing bus, portal or namespace is the
    /// defaults.
    pub async fn read(&self) -> SystemPrefs {
        match self {
            SystemPrefsSource::Fixed(prefs) => *prefs,
            SystemPrefsSource::Portal => bus::read_all().await.unwrap_or_default(),
        }
    }
}

/// A subscription to the desktop's appearance preferences.
#[derive(Debug)]
pub struct SystemPrefsWatch {
    changes: Receiver<SystemPrefs>,
}

impl SystemPrefsWatch {
    /// Read `source` and subscribe to it, the subscription's task running on `spawn`. A
    /// [`SystemPrefsSource::Fixed`] watch, a desktop without the portal and a build without the portal client
    /// never fire after their first answer.
    pub async fn start(source: &SystemPrefsSource, spawn: &dyn Spawner) -> Self {
        let (tx, changes) = latest::channel(source.read().await);
        match source {
            SystemPrefsSource::Fixed(_) => drop(tx),
            SystemPrefsSource::Portal => spawn.spawn(Box::pin(bus::follow(tx))),
        }
        SystemPrefsWatch { changes }
    }

    /// The preferences as last read.
    pub fn current(&self) -> SystemPrefs {
        self.changes.latest()
    }

    /// Wait for the next change. `None` once the bus connection is gone.
    pub async fn changed(&mut self) -> Option<SystemPrefs> {
        self.changes.changed().await
    }
}

#[cfg(test)]
mod tests {
    use super::{contrast_from_portal, reduced_motion_from_portal, scheme_from_portal};

    use super::SystemPrefsSource;
    use ds_style::appearance::system::Contrast;
    use ds_style::appearance::system::ReducedMotion;
    use ds_style::appearance::system::SystemPrefs;
    use ds_style::appearance::theme::Scheme;

    #[test]
    fn the_scheme_table_matches_the_portal_spec() {
        const CASES: &[(u32, Scheme)] = &[
            (0, Scheme::Light),
            (1, Scheme::Dark),
            (2, Scheme::Light),
            (99, Scheme::Light),
        ];
        for &(value, want) in CASES {
            assert_eq!(scheme_from_portal(value), want, "color-scheme {value}");
        }
    }

    #[test]
    fn the_reduced_motion_table_matches_the_portal_spec() {
        const CASES: &[(u32, ReducedMotion)] = &[
            (0, ReducedMotion::NoPreference),
            (1, ReducedMotion::Reduce),
            (7, ReducedMotion::NoPreference),
        ];
        for &(value, want) in CASES {
            assert_eq!(
                reduced_motion_from_portal(value),
                want,
                "reduced-motion {value}"
            );
        }
    }

    #[test]
    fn the_contrast_table_matches_the_portal_spec() {
        const CASES: &[(u32, Contrast)] = &[
            (0, Contrast::Normal),
            (1, Contrast::High),
            (5, Contrast::Normal),
        ];
        for &(value, want) in CASES {
            assert_eq!(contrast_from_portal(value), want, "contrast {value}");
        }
    }

    #[tokio::test]
    async fn a_fixed_source_answers_its_own_preferences_and_never_touches_the_bus() {
        let fixed = SystemPrefs {
            scheme: Scheme::Dark,
            motion: ReducedMotion::Reduce,
            contrast: Contrast::High,
        };
        assert_eq!(SystemPrefsSource::Fixed(fixed).read().await, fixed);
    }
}
