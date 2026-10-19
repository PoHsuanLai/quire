//! The portal's stand-in on macOS: the same two entry points as `portal/bus.rs`, answered from the
//! user's preferences. Dark is the global domain's `AppleInterfaceStyle`, which the system sets to
//! `Dark` while the Mac is dark (Auto included) and removes while it is light; reduced motion and
//! higher contrast are `com.apple.universalaccess`'s `reduceMotion` and `increaseContrast`.
//!
//! The system announces a change of appearance only as a distributed notification, which arrives
//! on a run loop; this crate owns no run loop, so the watch reads the preferences again every
//! [`EVERY`] and publishes only what changed. A read is three lookups in `cfprefsd`'s cache.

use super::SystemPrefs;
use crate::latest::{self, Receiver, Sender};
use ds_core::spawner::Spawner;
use ds_style::appearance::system::{Contrast, ReducedMotion};
use ds_style::appearance::theme::Scheme;
use objc2_core_foundation::{
    CFBoolean, CFNumber, CFPreferencesAppSynchronize, CFPreferencesCopyAppValue, CFPropertyList,
    CFRetained, CFString,
};
use std::time::Duration;

/// How often the watch reads the preferences again.
const EVERY: Duration = Duration::from_secs(1);

/// The global domain (`kCFPreferencesAnyApplication`, whose value is its own name: the constant
/// is an extern static, which would need `unsafe` to read).
const GLOBAL: &str = "kCFPreferencesAnyApplication";
const ACCESSIBILITY: &str = "com.apple.universalaccess";
const KEY_STYLE: &str = "AppleInterfaceStyle";
const KEY_REDUCE_MOTION: &str = "reduceMotion";
const KEY_INCREASE_CONTRAST: &str = "increaseContrast";

/// `AppleInterfaceStyle`: `Dark` is dark; anything else, or no value, is light.
fn scheme_from_style(style: Option<&str>) -> Scheme {
    match style {
        Some(style) if style.eq_ignore_ascii_case("dark") => Scheme::Dark,
        _ => Scheme::Light,
    }
}

/// The three answers, as the portal's tables would give them.
fn prefs_from(style: Option<&str>, reduce_motion: bool, increase_contrast: bool) -> SystemPrefs {
    SystemPrefs::default()
        .with_scheme(scheme_from_style(style))
        .with_motion(if reduce_motion {
            ReducedMotion::Reduce
        } else {
            ReducedMotion::NoPreference
        })
        .with_contrast(if increase_contrast {
            Contrast::High
        } else {
            Contrast::Normal
        })
}

/// `key` in `domain`, as it is now.
fn value(key: &str, domain: &CFString) -> Option<CFRetained<CFPropertyList>> {
    CFPreferencesCopyAppValue(&CFString::from_str(key), domain)
}

/// A switch in `domain`: a boolean, or a number that is not zero. Missing is off.
fn flag(key: &str, domain: &CFString) -> bool {
    let Some(value) = value(key, domain) else {
        return false;
    };
    if let Some(boolean) = value.downcast_ref::<CFBoolean>() {
        return boolean.as_bool();
    }
    value
        .downcast_ref::<CFNumber>()
        .and_then(CFNumber::as_i64)
        .is_some_and(|number| number != 0)
}

/// The preferences as the user's defaults hold them now; a domain written by another process
/// since the last read is read afresh.
fn read() -> SystemPrefs {
    let global = CFString::from_static_str(GLOBAL);
    let accessibility = CFString::from_static_str(ACCESSIBILITY);
    CFPreferencesAppSynchronize(&global);
    CFPreferencesAppSynchronize(&accessibility);
    let style = value(KEY_STYLE, &global)
        .and_then(|value| value.downcast_ref::<CFString>().map(ToString::to_string));
    prefs_from(
        style.as_deref(),
        flag(KEY_REDUCE_MOTION, &accessibility),
        flag(KEY_INCREASE_CONTRAST, &accessibility),
    )
}

/// Every preference, once.
pub(super) async fn read_all() -> Option<SystemPrefs> {
    Some(read())
}

/// Read again every [`EVERY`] and publish a change, until the receiver is gone.
async fn follow(tx: Sender<SystemPrefs>) {
    loop {
        futures_timer::Delay::new(EVERY).await;
        let now = read();
        if now != tx.latest() && tx.send(now).is_err() {
            return;
        }
    }
}

/// The preferences now and every change after, the watch running on `spawn`.
pub(super) async fn start(spawn: &dyn Spawner) -> Receiver<SystemPrefs> {
    let (tx, changes) = latest::channel(read());
    spawn.spawn(Box::pin(follow(tx)));
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_style_and_the_switches_map_as_the_portal_s_tables_do() {
        let cases: &[(&str, Option<&str>, bool, bool, SystemPrefs)] = &[
            (
                "light: no style",
                None,
                false,
                false,
                SystemPrefs::default(),
            ),
            (
                "dark",
                Some("Dark"),
                false,
                false,
                SystemPrefs::default().with_scheme(Scheme::Dark),
            ),
            (
                "an unknown style is light",
                Some("Graphite"),
                false,
                false,
                SystemPrefs::default(),
            ),
            (
                "both switches on",
                None,
                true,
                true,
                SystemPrefs::default()
                    .with_motion(ReducedMotion::Reduce)
                    .with_contrast(Contrast::High),
            ),
        ];
        for (name, style, motion, contrast, want) in cases {
            assert_eq!(prefs_from(*style, *motion, *contrast), *want, "{name}");
        }
    }
}
