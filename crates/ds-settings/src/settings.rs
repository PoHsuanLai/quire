//! The keys `appearance.toml` holds (design/22-SETTINGS.md sections 3.1-3.3 and 4.3-4.4).
//!
//! Every struct is `#[serde(default)]` and read through [`crate::lenient`], so a bad value costs
//! only its own key; every struct keeps the keys it does not know in `extra`, so a round trip
//! through an older or newer binary drops nothing.

use crate::schema::Page;
use crate::units::{Fraction, Percent};
use ds::{Accent, Appearance, Look, Motion, Theme, Warmth};
use serde::{Deserialize, Serialize};

/// `appearance.*`: what every surface resolves its look from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/appearance.toml", domain = "appearance", page = Page::Appearance)]
pub struct AppearanceSettings {
    /// `appearance.theme`: System, Light or Dark.
    #[settings(
        label = "Theme",
        help = "Follow the desktop, or force light or dark mode.",
        section = "Appearance"
    )]
    pub theme: Theme,
    /// `appearance.look`: Post unless the person picks another.
    #[settings(
        label = "Look",
        help = "The card's whole visual language: Post, Riso, Tide or Candy.",
        section = "Appearance"
    )]
    pub look: Look,
    /// `appearance.warmth`: applies only when the look is Candy.
    #[settings(
        label = "Warmth",
        help = "How warm Candy's neutrals run. Applies only when Look is Candy.",
        section = "Appearance"
    )]
    pub warmth: Warmth,
    /// `appearance.accent`: one of six.
    #[settings(
        label = "Accent",
        help = "The card's accent colour.",
        section = "Appearance"
    )]
    pub accent: Accent,
    /// `appearance.motion_level` (alias `motion.level`): System follows the portal.
    #[settings(
        label = "Motion",
        help = "How much the window moves: Calm, Standard, Extra, or reduce motion to a \
                single frame.",
        section = "Appearance"
    )]
    pub motion_level: Motion,
    /// `appearance.material_tint_alpha`: the tint's alpha over compositor blur (proposed 80).
    #[settings(
        label = "Material tint",
        help = "The tint's alpha over the compositor's behind-surface blur.",
        section = "Appearance",
        range = "0..=100",
        unit = "%",
        advanced
    )]
    pub material_tint_alpha: Percent,
    /// Keys this build does not know, kept for the next write.
    #[serde(flatten)]
    #[settings(skip)]
    pub extra: toml::Table,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        AppearanceSettings {
            theme: Theme::System,
            look: Look::Post,
            warmth: Warmth::Neutral,
            accent: Accent::Postmark,
            motion_level: Motion::System,
            material_tint_alpha: Percent(80),
            extra: toml::Table::new(),
        }
    }
}

impl AppearanceSettings {
    /// The three choices a `ds::Ds` root resolves.
    pub fn appearance(&self) -> Appearance {
        Appearance {
            theme: self.theme,
            accent: self.accent,
            motion: self.motion_level,
        }
    }
}

/// How a plate's glyph is coloured (`icons.plate_glyph_colour_policy`).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize, crate::SettingsSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PlateGlyphPolicy {
    /// WCAG-driven per family: red, blue, violet white; amber, green ink.
    #[default]
    Auto,
    /// Always white.
    ForceWhite,
    /// Always ink.
    ForceInk,
}

/// Whether third-party icons get a dark variant (`icons.dark_mode_variant`).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize, crate::SettingsSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum IconDarkVariant {
    /// The freedesktop convention: the same icon in both schemes.
    #[default]
    SameAsLight,
    /// A dark variant where the plate needs one.
    Adaptive,
}

/// `icons.*`: how the dock and launcher plate third-party icons (design/08-ICONS.md section 4).
///
/// All five keys are Advanced (design/22-SETTINGS.md section 5: not listed on any Basic page),
/// on the Appearance page since they live in `appearance.toml` alongside the `appearance`
/// domain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/appearance.toml", domain = "icons", page = Page::Appearance)]
pub struct IconsSettings {
    /// `icons.plate_inset_percent` (proposed 72).
    #[settings(
        label = "Plate inset",
        help = "How far a third-party icon sits inside its plate.",
        section = "Icons",
        range = "0..=100",
        unit = "%",
        advanced
    )]
    pub plate_inset_percent: Percent,
    /// `icons.symbolic_fallback_glyph_percent` (proposed 56).
    #[settings(
        label = "Symbolic fallback size",
        help = "The glyph's size when an app ships only a symbolic icon.",
        section = "Icons",
        range = "0..=100",
        unit = "%",
        advanced
    )]
    pub symbolic_fallback_glyph_percent: Percent,
    /// `icons.squircle_detect_iou` (proposed 0.90).
    #[settings(
        label = "Squircle detection",
        help = "How closely an icon must already match a squircle to skip re-masking it \
                (stored per-mille: 900 is 0.90).",
        section = "Icons",
        range = "0..=1000",
        advanced
    )]
    pub squircle_detect_iou: Fraction,
    /// `icons.plate_glyph_colour_policy`.
    #[settings(
        label = "Plate glyph colour",
        help = "Auto picks white or ink per family for contrast; the other two force one.",
        section = "Icons",
        advanced
    )]
    pub plate_glyph_colour_policy: PlateGlyphPolicy,
    /// `icons.dark_mode_variant`.
    #[settings(
        label = "Icon dark variant",
        help = "Whether a third-party icon adapts to the dark scheme or stays the same.",
        section = "Icons",
        advanced
    )]
    pub dark_mode_variant: IconDarkVariant,
    /// Keys this build does not know, kept for the next write.
    #[serde(flatten)]
    #[settings(skip)]
    pub extra: toml::Table,
}

impl Default for IconsSettings {
    fn default() -> Self {
        IconsSettings {
            plate_inset_percent: Percent(72),
            symbolic_fallback_glyph_percent: Percent(56),
            squircle_detect_iou: Fraction(900),
            plate_glyph_colour_policy: PlateGlyphPolicy::Auto,
            dark_mode_variant: IconDarkVariant::SameAsLight,
            extra: toml::Table::new(),
        }
    }
}

/// `quire/appearance.toml`, whole.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceFile {
    /// The schema version, 1. Nothing reads it yet but its absence.
    pub version: u16,
    /// `[appearance]`.
    pub appearance: AppearanceSettings,
    /// `[icons]`.
    pub icons: IconsSettings,
    /// Tables this build does not know, kept for the next write.
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl Default for AppearanceFile {
    fn default() -> Self {
        AppearanceFile {
            version: 1,
            appearance: AppearanceSettings::default(),
            icons: IconsSettings::default(),
            extra: toml::Table::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppearanceFile, AppearanceSettings, IconsSettings};
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    /// `CONVENTIONS.md#3-serde`: "every persisted type has a round-trip test." Raw
    /// `toml::to_string`/`from_str`, not through [`crate::file`] or [`crate::lenient`], so this
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
