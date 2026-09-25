//! Settings I/O: any settings file through one generic API (`file::{Settings, load, save}`,
//! lenient read, atomic write, a debounced directory watch), with `appearance.toml` (plus a
//! one-time import of mailo's `appearance.json`) and `spaces.json` as its two instances, and
//! the settings portal (design/22-SETTINGS.md sections 2 and 4, design/21-SPACES.md section 10).
//!
//! `ds` stays effect-free; everything here touches the disk or the bus.

// `#[derive(SettingsSchema)]`'s generated code always writes `::ds_settings::schema::...`, so
// that the same derive output resolves whether a consumer crate invokes it or this crate does
// (`settings.rs`'s own `AppearanceSettings`/`IconsSettings`). This self-referential alias is
// what lets the second case resolve too, without a second, crate-relative code path in
// `ds-settings-derive`.
extern crate self as ds_settings;

pub mod appearance_file;
pub mod dbus;
pub mod diff;
pub mod dirs;
pub mod environment;
pub mod error;
pub mod file;
pub mod icon_assets;
pub mod lenient;
pub mod portal;
pub mod schema;
pub mod settings;
pub mod spaces;
#[cfg(test)]
mod test_dir;
pub mod units;
pub mod watch;

pub use appearance_file::{APPEARANCE, FILE_NAME, load, load_or_import, save};
pub use diff::{SettingsChange, apply};
pub use dirs::{AppName, cache_dir, config_dir, state_dir};
pub use ds_settings_derive::SettingsSchema;
pub use environment::{Environment, use_environment};
pub use error::SettingsError;
pub use file::{FileName, Format, Settings, SettingsFile};
pub use icon_assets::{app_icon_path, apps_dir};
pub use lenient::{lenient, lenient_json};
pub use portal::{
    SystemPrefsWatch, contrast_from_portal, read_system_prefs, reduced_motion_from_portal,
    scheme_from_portal,
};
pub use settings::{
    AppearanceFile, AppearanceSettings, IconDarkVariant, IconStyle, IconsSettings, MonochromeTint,
    PlateGlyphPolicy,
};
pub use spaces::{SPACES, SpacesWatch};
pub use units::{Count, Fraction, Ms, Percent, Px, Scalar, Units};
pub use watch::{AppearanceWatch, DEBOUNCE, FileWatch, watch, watch_file};
