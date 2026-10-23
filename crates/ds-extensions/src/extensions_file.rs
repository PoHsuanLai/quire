//! `quire/extensions.toml`: which Extensions are turned off (design: Extensions, section 4.5).
//!
//! Off is stored and on is the default, so a new Extension works without a write. The file is an
//! ordinary schema'd settings file (page Extensions), layered like any setting, so an
//! administrator can turn an Extension off and lock it. Turning one on or off is the person's:
//! no key of it is agent-settable.

use ds_core::settings::ExtensionId;
use ds_settings::schema::{Page, Schema, SettingsSchema};
use ds_settings::{FileName, Format, SettingsDoc, SettingsRow, SettingsSchema};
use serde::{Deserialize, Serialize};

/// One Extension that is turned off, by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SettingsRow)]
pub struct OffRow {
    /// The Extension's id: dotted words of letters, digits, `-` and `_`.
    #[settings(label = "Extension", pattern = "[A-Za-z0-9_-]+(\\.[A-Za-z0-9_-]+)*")]
    pub id: String,
}

/// Whether an Extension is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Power {
    /// In use. The default for every Extension.
    On,
    /// Turned off by the person or an administrator.
    Off,
}

/// `extensions.*`: the keys of `extensions.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/extensions.toml", domain = "extensions", page = Page::declared("extensions"))]
pub struct ExtensionsSettings {
    /// `extensions.off`: the ids turned off.
    #[settings(
        label = "Turned off",
        help = "Extensions you turned off. They stay installed and do nothing until you turn \
                them on again.",
        section = "Extensions"
    )]
    pub off: Vec<OffRow>,
}

impl ExtensionsSettings {
    /// Whether `id` is turned off.
    pub fn is_off(&self, id: &ExtensionId) -> bool {
        self.off.iter().any(|row| row.id == id.0)
    }

    /// These settings with `id` turned `power`: an id appears once, and turning on removes it.
    pub fn with(&self, id: &ExtensionId, power: Power) -> ExtensionsSettings {
        let mut off: Vec<OffRow> = self
            .off
            .iter()
            .filter(|row| row.id != id.0)
            .cloned()
            .collect();
        if power == Power::Off {
            off.push(OffRow { id: id.0.clone() });
        }
        ExtensionsSettings { off }
    }
}

/// `quire/extensions.toml`, whole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtensionsFile {
    /// The schema version, 1. Nothing reads it yet but its absence.
    pub version: u16,
    /// `[extensions]`.
    pub extensions: ExtensionsSettings,
}

impl Default for ExtensionsFile {
    fn default() -> Self {
        ExtensionsFile {
            version: 1,
            extensions: ExtensionsSettings::default(),
        }
    }
}

impl ExtensionsFile {
    /// The ids turned off, as [`Search::scan`](crate::Search::scan) takes them. Blank ids are
    /// left out.
    pub fn off_ids(&self) -> Vec<ExtensionId> {
        self.extensions
            .off
            .iter()
            .map(|row| row.id.trim())
            .filter(|id| !id.is_empty())
            .map(|id| ExtensionId(id.to_owned()))
            .collect()
    }
}

impl SettingsDoc for ExtensionsFile {
    const FILE: FileName = FileName("extensions.toml");
    const FORMAT: Format = Format::Toml;
}

/// The schema of `quire/extensions.toml`: one `Rows` key, `extensions.off`.
pub fn extensions_schema() -> Schema {
    ExtensionsSettings::schema()
}
