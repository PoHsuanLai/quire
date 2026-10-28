//! The keys `appearance.toml` holds (design/22-SETTINGS.md sections 3.1-3.3 and 4.3-4.4).
//!
//! Every struct is `#[serde(default)]` and read through [`crate::Store`], so a bad value costs
//! only its own key; a key a struct does not read is reported on load and dropped on save.

use crate::region::RegionSettings;
use crate::schema::{Page, Provided};
use ds_core::word::Word;
use ds_style::appearance::accent::Accent;
use ds_style::appearance::appearance::Appearance;
use ds_style::appearance::motion::Motion;
use ds_style::appearance::theme::Theme;
use ds_style::appearance::transparency::Transparency;
use ds_style::kit::CustomStyles;
use serde::{Deserialize, Serialize};

/// `appearance.*`: what every surface resolves its look from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/appearance.toml", domain = "appearance", page = Page::Appearance)]
#[non_exhaustive]
pub struct AppearanceSettings {
    /// `appearance.theme`: System, Light or Dark.
    #[settings(
        label = "Theme",
        help = "Follows the desktop, or stays in light or dark mode.",
        section = "Colour",
        agent_settable
    )]
    pub theme: Theme,
    /// `appearance.accent`: one of macOS's eight.
    #[settings(
        label = "Accent",
        help = "Colours buttons, selection, switches and focus rings.",
        section = "Colour",
        agent_settable
    )]
    pub accent: Accent,
    /// `appearance.motion_level`: Standard, which the desktop's reduce-motion preference still
    /// overrides, or Reduced. Read as macOS's "Reduce motion": a switch that is on for Reduced.
    #[settings(
        label = "Reduce motion",
        help = "Swaps slides and springs for fades.",
        note = "The desktop's own reduce-motion setting always applies.",
        section = "Accessibility"
    )]
    pub motion_level: Motion,
    /// `appearance.transparency`: System keeps translucent materials, Reduced paints every
    /// material solid. Read as macOS's "Reduce transparency": a switch that is on for Reduced.
    /// The compositor reads it from this file, as it reads `theme`.
    #[settings(
        label = "Reduce transparency",
        help = "Makes blurred and see-through areas solid.",
        section = "Accessibility"
    )]
    pub transparency: Transparency,
    /// `appearance.look`: `standard`, or the id of an installed look Extension (a stylesheet
    /// that restyles every quire surface). A stored id that is not installed, or is turned off,
    /// reads as `standard` where the installed Extensions are known and is reported once.
    #[settings(
        label = "Look",
        help = "Restyles every window.",
        note = "Standard puts the desktop back as it shipped.",
        section = "Style",
        provided = "look",
        agent_settable
    )]
    pub look: Provided,
    /// `appearance.file_icons`: `standard`, or the id of an installed file-icons Extension (a
    /// folder of pictures keyed by file type). A stored id that is not installed, or is turned
    /// off, reads as `standard` where the installed Extensions are known and is reported once.
    #[settings(
        label = "File icons",
        help = "Shows icons you added for each file type.",
        note = "A file type without an icon keeps the standard one.",
        section = "Icons",
        provided = "file_icons",
        agent_settable
    )]
    pub file_icons: Provided,
    /// `appearance.custom_styles`: whether the look and the person's own stylesheets are drawn.
    /// `off` is the way back from a style that broke a surface; it is a switch, "Use my styles".
    #[settings(
        label = "Use my styles",
        help = "Draws the look you chose and your own style sheets.",
        note = "Turn this off if a style makes something hard to use.",
        section = "Style",
        agent_settable
    )]
    pub custom_styles: CustomStyles,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        AppearanceSettings {
            theme: Theme::System,
            accent: Accent::Blue,
            motion_level: Motion::Standard,
            transparency: Transparency::System,
            look: Provided::standard(),
            file_icons: Provided::standard(),
            custom_styles: CustomStyles::Use,
        }
    }
}

impl AppearanceSettings {
    /// The choices a `ds::prelude::Ds` root resolves.
    pub fn appearance(&self) -> Appearance {
        Appearance {
            theme: self.theme,
            accent: self.accent,
            motion: self.motion_level,
            transparency: self.transparency,
        }
    }
}

/// Which dialect our app icons are drawn in (`icons.style`, design/08-ICONS.md section 2.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
pub enum IconStyle {
    /// Each app in the dialect its icon was designed in.
    #[default]
    Colour,
    /// The same icons with their colour held further down (a lower chroma cap).
    Muted,
    /// Every icon in one hue, tone on tone; third-party icons desaturated and re-tinted to match.
    Monochrome,
}

/// `icons.*`: how our app icons are drawn (design/08-ICONS.md section 2.10).
///
/// Only `style` is a setting. The plate's inset and glyph shares are the `--plate-*` tokens
/// (`ds_style::icon::family::PlateShare`), the symbolic chroma limit is
/// `ChromaLimit::PROPOSED`, and a Monochrome icon takes its hue from the Space (design/08).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/appearance.toml", domain = "icons", page = Page::Appearance)]
#[non_exhaustive]
pub struct IconsSettings {
    /// `icons.style` (proposed Colour).
    #[settings(
        label = "App icon style",
        help = "How colourful app icons are.",
        note = "Colour keeps each icon as designed; Muted holds the colour down; Monochrome uses one hue.",
        section = "Icons"
    )]
    pub style: IconStyle,
}

impl Default for IconsSettings {
    fn default() -> Self {
        IconsSettings {
            style: IconStyle::Colour,
        }
    }
}

/// `quire/appearance.toml`, whole.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct AppearanceFile {
    /// The schema version, 1. Nothing reads it yet but its absence.
    pub version: u16,
    /// `[appearance]`.
    pub appearance: AppearanceSettings,
    /// `[icons]`.
    pub icons: IconsSettings,
    /// `[region]`.
    pub region: RegionSettings,
}

impl Default for AppearanceFile {
    fn default() -> Self {
        AppearanceFile {
            version: 1,
            appearance: AppearanceSettings::default(),
            icons: IconsSettings::default(),
            region: RegionSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppearanceFile, AppearanceSettings, IconsSettings};
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    /// `CONVENTIONS.md#12-derives-and-serde`: "every stored or wire type has a round-trip test."
    /// Raw `toml::to_string`/`from_str`, not through [`crate::Store`], so this
    /// exercises the struct's own `Serialize`/`Deserialize` in isolation.
    fn round_trips<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Default>() {
        let default = T::default();
        let text = toml::to_string(&default).unwrap_or_else(|e| panic!("{e}"));
        let back: T = toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(back, default, "{text}");
    }

    #[test]
    fn appearance_settings_round_trips() {
        round_trips::<AppearanceSettings>();
    }

    #[test]
    fn icons_settings_round_trips() {
        round_trips::<IconsSettings>();
    }

    #[test]
    fn appearance_file_round_trips() {
        round_trips::<AppearanceFile>();
    }
}
