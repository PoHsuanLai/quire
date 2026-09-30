//! `quire/appearance.toml` (design/22-SETTINGS.md sections 3.1-3.3 and 4.3-4.4): the file as a
//! [`crate::SettingsDoc`], and the settings structs it holds.

mod file;
mod schema;
mod settings;

pub use schema::quire_schema;
pub use settings::{
    AppearanceFile, AppearanceSettings, IconDarkVariant, IconStyle, IconsSettings, MonochromeTint,
    PlateGlyphPolicy,
};
