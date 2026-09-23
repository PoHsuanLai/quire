//! The desktop's preferences from the settings portal:
//! `org.freedesktop.portal.Settings.ReadAll(["org.freedesktop.appearance"])` and
//! `SettingChanged`, mapped to [`ds::SystemPrefs`]. The mappings are pure tables.
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use crate::error::SettingsError;
use ds::{Contrast, ReducedMotion, Scheme, SystemPrefs};

/// `color-scheme`: 0 no preference, 1 prefer dark, 2 prefer light. No preference is light.
pub fn scheme_from_portal(value: u32) -> Scheme {
    todo!()
}

/// `reduced-motion`: 0 no preference, 1 reduce.
pub fn reduced_motion_from_portal(value: u32) -> ReducedMotion {
    todo!()
}

/// `contrast`: 0 no preference, 1 higher contrast.
pub fn contrast_from_portal(value: u32) -> Contrast {
    todo!()
}

/// Read every appearance preference once. A desktop without the portal answers the defaults.
pub async fn read_system_prefs() -> Result<SystemPrefs, SettingsError> {
    todo!()
}

/// A subscription to `SettingChanged` for the appearance namespace.
#[derive(Debug)]
pub struct SystemPrefsWatch {
    changes: tokio::sync::watch::Receiver<SystemPrefs>,
}

impl SystemPrefsWatch {
    /// Subscribe.
    pub async fn start() -> Result<Self, SettingsError> {
        todo!()
    }

    /// Wait for the next change. `None` once the bus connection is gone.
    pub async fn changed(&mut self) -> Option<SystemPrefs> {
        todo!()
    }
}
