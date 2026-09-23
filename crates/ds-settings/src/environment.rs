//! Everything a surface resolves its look from, as one live signal: the settings file and the
//! desktop's preferences, both watched.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::dirs::AppName;
use crate::settings::AppearanceFile;
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

/// Load `app`'s settings (importing mailo's JSON once), read the portal, and keep both live.
pub fn use_environment(app: AppName) -> ReadSignal<Environment> {
    todo!()
}
