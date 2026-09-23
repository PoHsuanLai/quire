//! Appearance settings I/O: `appearance.toml` (atomic write, lenient read, a one-time import of
//! mailo's `appearance.json`), a directory watch, and the settings portal
//! (design/22-SETTINGS.md sections 2 and 4).
//!
//! `ds` stays effect-free; everything here touches the disk or the bus.

pub mod dbus;
pub mod diff;
pub mod dirs;
pub mod environment;
pub mod error;
pub mod file;
pub mod lenient;
pub mod portal;
pub mod settings;
pub mod units;
pub mod watch;

pub use diff::{SettingsChange, apply};
pub use dirs::{AppName, cache_dir, config_dir, state_dir};
pub use environment::{Environment, use_environment};
pub use error::SettingsError;
pub use file::{FILE_NAME, load, load_or_import, save};
pub use lenient::lenient;
pub use portal::{
    SystemPrefsWatch, contrast_from_portal, read_system_prefs, reduced_motion_from_portal,
    scheme_from_portal,
};
pub use settings::{
    AppearanceFile, AppearanceSettings, IconDarkVariant, IconsSettings, PlateGlyphPolicy,
};
pub use units::{Count, Fraction, Ms, Percent, Px, Scalar, Units};
pub use watch::{AppearanceWatch, DEBOUNCE, watch};
