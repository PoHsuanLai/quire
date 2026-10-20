//! The per-app identity record: the names one app is known by.
//!
//! An app is a desktop id for the launcher, an `app_id` for the compositor, bus names for D-Bus
//! activation and an app name in its intents manifest. They are often different strings, and
//! code that guesses one from another goes wrong. The app declares them in its desktop entry, in
//! `X-Quire-` keys, and this is the one reader.

use super::desktop_id::DesktopId;
use crate::DesktopGroup;

/// `X-Quire-WindowId=`: the `app_id` the app's windows carry.
const WINDOW_ID_KEY: &str = "X-Quire-WindowId";
/// `X-Quire-BusNames=`: a `;` list of the well-known names the app owns on the session bus.
const BUS_NAMES_KEY: &str = "X-Quire-BusNames";
/// `X-Quire-IntentsApp=`: the app name in the app's intents manifest.
const INTENTS_APP_KEY: &str = "X-Quire-IntentsApp";
/// `StartupWMClass=`: what a window id defaults to.
const WM_CLASS_KEY: &str = "StartupWMClass";

/// The names of one installed app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppIdentity {
    /// The entry's desktop id: what the launcher starts and the dock keeps.
    pub desktop_id: DesktopId,
    /// The `app_id` its windows carry: `X-Quire-WindowId`, else `StartupWMClass`, else the
    /// desktop id without its suffix.
    pub window_id: String,
    /// The well-known session-bus names it owns; empty when it declares none.
    pub bus_names: Vec<String>,
    /// Its app name in the intents manifest; none when it has no manifest.
    pub intents_app: Option<String>,
}

impl AppIdentity {
    /// The identity the `[Desktop Entry]` group `entry` declares for the file `desktop_id`.
    /// Nothing here can fail: a key that is missing or blank falls back or stays empty.
    pub fn from_entry(desktop_id: DesktopId, entry: &DesktopGroup) -> AppIdentity {
        let text = |key: &str| {
            entry
                .string(key)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        };
        let window_id = text(WINDOW_ID_KEY)
            .or_else(|| text(WM_CLASS_KEY))
            .unwrap_or_else(|| desktop_id.stem().to_owned());
        AppIdentity {
            window_id,
            bus_names: entry
                .list(BUS_NAMES_KEY)
                .into_iter()
                .map(|name| name.trim().to_owned())
                .filter(|name| !name.is_empty())
                .collect(),
            intents_app: text(INTENTS_APP_KEY),
            desktop_id,
        }
    }
}
