//! The desktop's preferences from the settings portal:
//! `org.freedesktop.portal.Settings.ReadAll(["org.freedesktop.appearance"])` and
//! `SettingChanged`, mapped to [`ds::SystemPrefs`]. The mappings are pure tables, tested
//! against a fake `ReadAll`/`SettingChanged` payload with no bus involved; only [`read_system_prefs`]
//! and [`SystemPrefsWatch`] touch `zbus`, and only on Linux — a desktop without the portal, or a
//! build for a platform that has none, simply answers [`ds::SystemPrefs::default`].

use crate::error::SettingsError;
use ds::{Contrast, ReducedMotion, Scheme, SystemPrefs};
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

/// The one namespace `ds-settings` reads; the portal groups every key by namespace, and
/// appearance is the only one this crate resolves (design/22-SETTINGS.md section 4.4).
const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY_COLOR_SCHEME: &str = "color-scheme";
const KEY_CONTRAST: &str = "contrast";
const KEY_REDUCED_MOTION: &str = "reduced-motion";

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

/// A key's raw `u32`, or `None` for a value that is missing or not a `u32` — a portal that
/// answers something unexpected is treated the same as a missing key (section 2's leniency
/// applies here too), never an error.
fn as_u32(value: &OwnedValue) -> Option<u32> {
    u32::try_from(value.clone()).ok()
}

/// [`SystemPrefs`] from one `ReadAll` namespace's key/value map. Every key is independently
/// optional: the desktop may answer some and not others, and each missing or malformed key
/// falls back to its own default, never the whole struct's.
fn prefs_from_namespace(namespace: &HashMap<String, OwnedValue>) -> SystemPrefs {
    SystemPrefs {
        scheme: namespace
            .get(KEY_COLOR_SCHEME)
            .and_then(as_u32)
            .map(scheme_from_portal)
            .unwrap_or_default(),
        motion: namespace
            .get(KEY_REDUCED_MOTION)
            .and_then(as_u32)
            .map(reduced_motion_from_portal)
            .unwrap_or_default(),
        contrast: namespace
            .get(KEY_CONTRAST)
            .and_then(as_u32)
            .map(contrast_from_portal)
            .unwrap_or_default(),
    }
}

/// Fold one `SettingChanged(namespace, key, value)` onto `prefs`; a namespace or key this crate
/// does not read, or a value that is not a `u32`, leaves `prefs` unchanged.
fn apply_setting_changed(
    prefs: SystemPrefs,
    namespace: &str,
    key: &str,
    value: &OwnedValue,
) -> SystemPrefs {
    if namespace != NAMESPACE {
        return prefs;
    }
    let Some(raw) = as_u32(value) else {
        return prefs;
    };
    match key {
        KEY_COLOR_SCHEME => SystemPrefs {
            scheme: scheme_from_portal(raw),
            ..prefs
        },
        KEY_REDUCED_MOTION => SystemPrefs {
            motion: reduced_motion_from_portal(raw),
            ..prefs
        },
        KEY_CONTRAST => SystemPrefs {
            contrast: contrast_from_portal(raw),
            ..prefs
        },
        _ => prefs,
    }
}

/// Read every appearance preference once. A desktop without the portal, or a non-Linux build,
/// answers the defaults; this never surfaces as an error the caller has to handle.
pub async fn read_system_prefs() -> Result<SystemPrefs, SettingsError> {
    #[cfg(target_os = "linux")]
    {
        Ok(bus::read_all().await.unwrap_or_default())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(SystemPrefs::default())
    }
}

/// A subscription to `SettingChanged` for the appearance namespace.
#[derive(Debug)]
pub struct SystemPrefsWatch {
    changes: tokio::sync::watch::Receiver<SystemPrefs>,
}

impl SystemPrefsWatch {
    /// Subscribe. A desktop without the portal, or a non-Linux build, yields a watch that never
    /// fires again after its (default) initial value — never an error.
    pub async fn start() -> Result<Self, SettingsError> {
        let initial = read_system_prefs().await?;
        let (tx, rx) = tokio::sync::watch::channel(initial);
        #[cfg(target_os = "linux")]
        bus::spawn_watch(tx);
        #[cfg(not(target_os = "linux"))]
        drop(tx);
        Ok(SystemPrefsWatch { changes: rx })
    }

    /// Wait for the next change. `None` once the bus connection is gone.
    pub async fn changed(&mut self) -> Option<SystemPrefs> {
        if self.changes.changed().await.is_err() {
            return None;
        }
        Some(*self.changes.borrow_and_update())
    }
}

/// The half of this module that actually reaches `zbus`; built only on Linux, the only platform
/// that ships `xdg-desktop-portal`.
#[cfg(target_os = "linux")]
mod bus {
    use super::{NAMESPACE, SystemPrefs, apply_setting_changed, prefs_from_namespace};
    use std::collections::HashMap;
    use zbus::export::ordered_stream::OrderedStreamExt;
    use zbus::proxy;
    use zbus::zvariant::OwnedValue;

    #[proxy(
        interface = "org.freedesktop.portal.Settings",
        default_service = "org.freedesktop.portal.Desktop",
        default_path = "/org/freedesktop/portal/desktop"
    )]
    trait Settings {
        #[zbus(name = "ReadAll")]
        fn read_all(
            &self,
            namespaces: &[&str],
        ) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>>;

        #[zbus(signal, name = "SettingChanged")]
        fn setting_changed(
            &self,
            namespace: String,
            key: String,
            value: OwnedValue,
        ) -> zbus::Result<()>;
    }

    /// `ReadAll(["org.freedesktop.appearance"])`, or `None` when the bus, the portal or the
    /// namespace is absent — every failure mode collapses to "use the defaults" at the caller.
    pub(super) async fn read_all() -> Option<SystemPrefs> {
        let connection = zbus::Connection::session().await.ok()?;
        let proxy = SettingsProxy::new(&connection).await.ok()?;
        let all = proxy.read_all(&[NAMESPACE]).await.ok()?;
        Some(
            all.get(NAMESPACE)
                .map(prefs_from_namespace)
                .unwrap_or_default(),
        )
    }

    /// Subscribe to `SettingChanged` and fold every appearance-namespace change onto `tx`. A
    /// desktop without the portal simply never sends anything; `tx` (and so the watch's
    /// receiver) is left at its initial value when the connection or subscription cannot be
    /// made.
    pub(super) fn spawn_watch(tx: tokio::sync::watch::Sender<SystemPrefs>) {
        tokio::spawn(async move {
            let Ok(connection) = zbus::Connection::session().await else {
                return;
            };
            let Ok(proxy) = SettingsProxy::new(&connection).await else {
                return;
            };
            let Ok(mut changes) = proxy.receive_setting_changed().await else {
                return;
            };
            while let Some(signal) = changes.next().await {
                let Ok(args) = signal.args() else {
                    continue;
                };
                let next =
                    apply_setting_changed(*tx.borrow(), args.namespace(), args.key(), args.value());
                if tx.send(next).is_err() {
                    return;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NAMESPACE, apply_setting_changed, contrast_from_portal, prefs_from_namespace,
        read_system_prefs, reduced_motion_from_portal, scheme_from_portal,
    };

    /// The real thing, against whatever settings portal this machine has (or does not).
    /// `read_system_prefs` never errors — a missing bus or portal is the defaults — so this
    /// mostly exists to be run by hand and its answer read: `cargo test -p ds-settings --
    /// --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "needs a session bus with the settings portal"]
    async fn the_live_portal_answers_something() {
        let prefs = read_system_prefs().await.unwrap_or_else(|e| panic!("{e}"));
        eprintln!("live settings portal answered: {prefs:?}");
    }
    use ds::{Contrast, ReducedMotion, Scheme, SystemPrefs};
    use std::collections::HashMap;
    use zbus::zvariant::OwnedValue;

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

    fn namespace(entries: &[(&str, u32)]) -> HashMap<String, OwnedValue> {
        entries
            .iter()
            .map(|&(key, value)| (key.to_owned(), OwnedValue::from(value)))
            .collect()
    }

    #[test]
    fn prefs_from_namespace_reads_every_key_independently() {
        let cases: &[(&str, HashMap<String, OwnedValue>, SystemPrefs)] = &[
            (
                "an empty answer is every default",
                namespace(&[]),
                SystemPrefs::default(),
            ),
            (
                "every key present",
                namespace(&[("color-scheme", 1), ("reduced-motion", 1), ("contrast", 1)]),
                SystemPrefs {
                    scheme: Scheme::Dark,
                    motion: ReducedMotion::Reduce,
                    contrast: Contrast::High,
                },
            ),
            (
                "only the scheme, the rest missing",
                namespace(&[("color-scheme", 1)]),
                SystemPrefs {
                    scheme: Scheme::Dark,
                    motion: ReducedMotion::NoPreference,
                    contrast: Contrast::Normal,
                },
            ),
            (
                "a key this crate does not read is ignored",
                namespace(&[("accent-color", 1), ("contrast", 1)]),
                SystemPrefs {
                    scheme: Scheme::Light,
                    motion: ReducedMotion::NoPreference,
                    contrast: Contrast::High,
                },
            ),
        ];
        for (name, given, want) in cases {
            assert_eq!(&prefs_from_namespace(given), want, "{name}");
        }
    }

    #[test]
    fn a_setting_changed_signal_updates_only_its_own_key() {
        let start = SystemPrefs::default();
        let cases: &[(&str, &str, &str, u32, SystemPrefs)] = &[
            (
                "a matching namespace and key",
                NAMESPACE,
                "color-scheme",
                1,
                SystemPrefs {
                    scheme: Scheme::Dark,
                    ..start
                },
            ),
            (
                "a different namespace is ignored",
                "org.gnome.desktop",
                "color-scheme",
                1,
                start,
            ),
            (
                "a key this crate does not read is ignored",
                NAMESPACE,
                "accent-color",
                1,
                start,
            ),
        ];
        for (name, namespace, key, raw, want) in cases {
            let got = apply_setting_changed(start, namespace, key, &OwnedValue::from(*raw));
            assert_eq!(&got, want, "{name}");
        }
    }
}
