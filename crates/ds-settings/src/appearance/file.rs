//! `appearance.toml` as a settings document.

use super::{AppearanceFile, quire_schema};
use crate::doc::{FileName, Format, SettingsDoc};
use crate::lenient::Limits;

impl SettingsDoc for AppearanceFile {
    const FILE: FileName = FileName("appearance.toml");
    const FORMAT: Format = Format::Toml;

    fn limits() -> Limits {
        Limits::of_schema(&quire_schema())
    }
}
