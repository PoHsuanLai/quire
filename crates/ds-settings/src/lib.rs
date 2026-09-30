//! Settings I/O: any settings file through one generic API ([`SettingsDoc`] and [`Store`]: lenient
//! read, atomic write, a debounced directory watch), with `appearance.toml` as its instance,
//! the settings portal, and the icon-theme lookup (design/22-SETTINGS.md sections 2 and 4).
//!
//! `ds` stays effect-free; everything here touches the disk or the bus. A watch or the portal
//! runs as a task on the [`ds_core::spawner::Spawner`] it is handed: no runtime is named here. Feature `dioxus`
//! adds [`use_environment`], the one signal a surface resolves its look from.

// `#[derive(SettingsSchema)]`'s generated code always writes `::ds_settings::schema::...`, so
// that the same derive output resolves whether a consumer crate invokes it or this crate does
// (`appearance/settings.rs`'s own `AppearanceSettings`/`IconsSettings`). This self-referential
// alias is what lets the second case resolve too, without a second, crate-relative code path in
// `ds-settings-derive`.
extern crate self as ds_settings;

mod appearance;
mod doc;
#[cfg(feature = "dioxus")]
mod environment;
mod error;
pub mod icon_assets;
mod latest;
mod lenient;
mod portal;
mod root;
pub mod schema;
mod store;
mod units;
mod watch;

pub use appearance::{
    AppearanceFile, AppearanceSettings, IconDarkVariant, IconStyle, IconsSettings, MonochromeTint,
    PlateGlyphPolicy,
};
pub use doc::{FileName, Format, SettingsDoc};
pub use ds_settings_derive::SettingsSchema;
#[cfg(feature = "dioxus")]
pub use environment::{Environment, use_environment};
pub use error::SettingsError;
pub use icon_assets::{app_icon_path, apps_dir};
pub use lenient::{InvalidKey, Loaded, UnknownKey};
pub use portal::{
    SystemPrefsSource, SystemPrefsWatch, contrast_from_portal, reduced_motion_from_portal,
    scheme_from_portal,
};
pub use root::{AppName, ConfigRoot};
pub use store::Store;
pub use units::{Count, Fraction, Mins, Ms, Percent, Px, Scalar, Secs, Units};
pub use watch::{DEBOUNCE, Watch, WatchState};
