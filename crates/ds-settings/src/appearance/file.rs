//! `appearance.toml` as a settings document.

use super::AppearanceFile;
use crate::doc::{FileName, Format, SettingsDoc};

impl SettingsDoc for AppearanceFile {
    const FILE: FileName = FileName("appearance.toml");
    const FORMAT: Format = Format::Toml;
}
