//! The portal client: the half of `portal.rs` that reaches `zbus`. Built only on Linux with feature
//! `quire-desktop`; `portal/absent.rs` answers the same two entry points elsewhere.

use super::{SystemPrefs, contrast_from_portal, reduced_motion_from_portal, scheme_from_portal};
use crate::latest::Sender;
use std::collections::HashMap;
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::proxy;
use zbus::zvariant::OwnedValue;

/// The one namespace `ds-settings` reads; the portal groups every key by namespace, and
/// appearance is the only one this crate resolves (design/22-SETTINGS.md section 4.4).
const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY_COLOR_SCHEME: &str = "color-scheme";
const KEY_CONTRAST: &str = "contrast";
const KEY_REDUCED_MOTION: &str = "reduced-motion";

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
    SystemPrefs::default()
        .with_scheme(
            namespace
                .get(KEY_COLOR_SCHEME)
                .and_then(as_u32)
                .map(scheme_from_portal)
                .unwrap_or_default(),
        )
        .with_motion(
            namespace
                .get(KEY_REDUCED_MOTION)
                .and_then(as_u32)
                .map(reduced_motion_from_portal)
                .unwrap_or_default(),
        )
        .with_contrast(
            namespace
                .get(KEY_CONTRAST)
                .and_then(as_u32)
                .map(contrast_from_portal)
                .unwrap_or_default(),
        )
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
        KEY_COLOR_SCHEME => prefs.with_scheme(scheme_from_portal(raw)),
        KEY_REDUCED_MOTION => prefs.with_motion(reduced_motion_from_portal(raw)),
        KEY_CONTRAST => prefs.with_contrast(contrast_from_portal(raw)),
        _ => prefs,
    }
}

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
/// desktop without the portal simply never sends anything: `tx` (and so the watch's
/// receiver) is left at its initial value when the connection or subscription cannot be made.
pub(super) async fn follow(tx: Sender<SystemPrefs>) {
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
        let next = apply_setting_changed(tx.latest(), args.namespace(), args.key(), args.value());
        if tx.send(next).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds_style::appearance::system::{Contrast, ReducedMotion};
    use ds_style::appearance::theme::Scheme;

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
                SystemPrefs::default()
                    .with_scheme(Scheme::Dark)
                    .with_motion(ReducedMotion::Reduce)
                    .with_contrast(Contrast::High),
            ),
            (
                "only the scheme, the rest missing",
                namespace(&[("color-scheme", 1)]),
                SystemPrefs::default()
                    .with_scheme(Scheme::Dark)
                    .with_motion(ReducedMotion::NoPreference)
                    .with_contrast(Contrast::Normal),
            ),
            (
                "a key this crate does not read is ignored",
                namespace(&[("accent-color", 1), ("contrast", 1)]),
                SystemPrefs::default()
                    .with_scheme(Scheme::Light)
                    .with_motion(ReducedMotion::NoPreference)
                    .with_contrast(Contrast::High),
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
                start.with_scheme(Scheme::Dark),
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
