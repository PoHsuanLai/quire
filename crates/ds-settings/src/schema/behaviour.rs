//! The settings of the machines in `ds-behaviour`: `switcher.*` (design/22 §3.11; design/13
//! §13.3.5) and `hot_corners.*` (design/13 §13.3.12). They live here, beside the schema, so the
//! shell and the compositor read one definition of each key; the keys stay in
//! `sill/settings.toml`. All Advanced.

use std::time::Duration;

use ds_behaviour::hot_corner::CornerParams;
use ds_behaviour::switcher::SwitcherParams;
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

use crate::schema::Page;
use crate::units::{Ms, Px};

/// `switcher.*`: the app switcher's timing and cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "sill/settings.toml", domain = "switcher", page = Page::KeyboardAndShortcuts)]
#[non_exhaustive]
pub struct SwitcherSettings {
    #[settings(
        label = "Show delay",
        help = "How long the chord is held before the switcher appears.",
        section = "App switcher",
        range = "0..=500",
        unit = "ms",
        advanced
    )]
    pub show_delay_ms: Ms,
    #[settings(
        label = "Icon size",
        help = "The size of an app icon in the switcher.",
        section = "App switcher",
        range = "32..=256",
        unit = "px",
        advanced
    )]
    pub icon_size_px: Px,
    #[settings(
        label = "Cell size",
        help = "The size of one app's cell.",
        section = "App switcher",
        range = "48..=256",
        unit = "px",
        advanced
    )]
    pub cell_size_px: Px,
    #[settings(
        label = "Cell gap",
        help = "Space between cells.",
        section = "App switcher",
        range = "0..=32",
        unit = "px",
        advanced
    )]
    pub cell_gap_px: Px,
    #[settings(
        label = "Smallest icon",
        help = "How small icons may shrink with many apps open.",
        section = "App switcher",
        range = "16..=128",
        unit = "px",
        advanced
    )]
    pub overflow_min_icon_px: Px,
}

impl Default for SwitcherSettings {
    fn default() -> Self {
        SwitcherSettings {
            show_delay_ms: Ms(150),
            icon_size_px: Px(96),
            cell_size_px: Px(112),
            cell_gap_px: Px(8),
            overflow_min_icon_px: Px(48),
        }
    }
}

impl SwitcherSettings {
    /// The switcher machine's timing.
    pub fn params(&self) -> SwitcherParams {
        SwitcherParams {
            show_delay: millis(self.show_delay_ms),
        }
    }
}

/// What a hot corner does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
#[non_exhaustive]
pub enum CornerAction {
    /// Nothing; the corner is inactive.
    #[default]
    None,
    /// Open the launcher.
    Launcher,
    /// Open the notification center.
    NotificationCenter,
    /// Open the control center.
    ControlCenter,
    /// Minimise every window of the Space; the corner again brings them back.
    ShowDesktop,
    /// Lock the screen.
    Lock,
    /// The overview of the Spaces.
    Workspaces,
    /// The corner's `*_command`, run by `/bin/sh -c`.
    Command,
}

/// `hot_corners.*`: what each screen corner does when the pointer rests in it, and the timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "sill/settings.toml", domain = "hot_corners", page = Page::Dock)]
#[non_exhaustive]
pub struct HotCornerSettings {
    #[settings(
        label = "Top left",
        help = "What the top-left corner does.",
        section = "Hot Corners",
        advanced
    )]
    pub top_left: CornerAction,
    #[settings(
        label = "Top right",
        help = "What the top-right corner does.",
        section = "Hot Corners",
        advanced
    )]
    pub top_right: CornerAction,
    #[settings(
        label = "Bottom left",
        help = "What the bottom-left corner does.",
        section = "Hot Corners",
        advanced
    )]
    pub bottom_left: CornerAction,
    #[settings(
        label = "Bottom right",
        help = "What the bottom-right corner does.",
        section = "Hot Corners",
        advanced
    )]
    pub bottom_right: CornerAction,
    #[settings(
        label = "Top-left command",
        help = "The command the top-left corner runs when it is set to Command.",
        section = "Hot Corners",
        advanced
    )]
    pub top_left_command: String,
    #[settings(
        label = "Top-right command",
        help = "The command the top-right corner runs when it is set to Command.",
        section = "Hot Corners",
        advanced
    )]
    pub top_right_command: String,
    #[settings(
        label = "Bottom-left command",
        help = "The command the bottom-left corner runs when it is set to Command.",
        section = "Hot Corners",
        advanced
    )]
    pub bottom_left_command: String,
    #[settings(
        label = "Bottom-right command",
        help = "The command the bottom-right corner runs when it is set to Command.",
        section = "Hot Corners",
        advanced
    )]
    pub bottom_right_command: String,
    #[settings(
        label = "Corner delay",
        help = "How long the pointer rests in a corner before it acts.",
        section = "Hot Corners",
        range = "0..=2000",
        unit = "ms",
        advanced
    )]
    pub dwell_ms: Ms,
    #[settings(
        label = "Corner re-arm time",
        help = "After acting, a corner acts again only once the pointer has left it and this long has passed.",
        section = "Hot Corners",
        range = "0..=5000",
        unit = "ms",
        advanced
    )]
    pub rearm_ms: Ms,
    #[settings(
        label = "Corner size",
        help = "The side of the invisible square in each active corner.",
        section = "Hot Corners",
        range = "1..=8",
        unit = "px",
        advanced
    )]
    pub size_px: Px,
}

impl Default for HotCornerSettings {
    fn default() -> Self {
        HotCornerSettings {
            top_left: CornerAction::None,
            top_right: CornerAction::None,
            bottom_left: CornerAction::None,
            bottom_right: CornerAction::None,
            top_left_command: String::new(),
            top_right_command: String::new(),
            bottom_left_command: String::new(),
            bottom_right_command: String::new(),
            dwell_ms: Ms(150),
            rearm_ms: Ms(500),
            size_px: Px(2),
        }
    }
}

impl HotCornerSettings {
    /// The corner machine's timing (the same for every corner).
    pub fn params(&self) -> CornerParams {
        CornerParams {
            dwell: millis(self.dwell_ms),
            rearm: millis(self.rearm_ms),
        }
    }
}

/// A key's milliseconds as a `Duration`.
fn millis(ms: Ms) -> Duration {
    Duration::from_millis(u64::from(ms.0))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ds_behaviour::hot_corner::CornerParams;
    use ds_behaviour::switcher::SwitcherParams;
    use ds_core::word::Word;

    use super::{CornerAction, HotCornerSettings, SwitcherSettings};
    use crate::units::Ms;

    #[test]
    fn params_carry_the_keys_milliseconds() {
        let switcher = SwitcherSettings {
            show_delay_ms: Ms(220),
            ..SwitcherSettings::default()
        };
        assert_eq!(
            switcher.params(),
            SwitcherParams {
                show_delay: Duration::from_millis(220)
            }
        );
        let corners = HotCornerSettings {
            dwell_ms: Ms(300),
            rearm_ms: Ms(1200),
            ..HotCornerSettings::default()
        };
        assert_eq!(
            corners.params(),
            CornerParams {
                dwell: Duration::from_millis(300),
                rearm: Duration::from_millis(1200)
            }
        );
    }

    #[test]
    fn defaults_are_design_13s_numbers() {
        assert_eq!(
            SwitcherSettings::default().params().show_delay,
            Duration::from_millis(150)
        );
        assert_eq!(
            HotCornerSettings::default().params(),
            CornerParams {
                dwell: Duration::from_millis(150),
                rearm: Duration::from_millis(500)
            }
        );
    }

    #[test]
    fn settings_round_trip_through_toml() {
        let switcher = SwitcherSettings {
            show_delay_ms: Ms(90),
            ..SwitcherSettings::default()
        };
        let text = toml::to_string(&switcher).expect("serialises");
        assert_eq!(
            toml::from_str::<SwitcherSettings>(&text).ok(),
            Some(switcher)
        );
        let corners = HotCornerSettings {
            top_left: CornerAction::Launcher,
            bottom_right: CornerAction::Command,
            bottom_right_command: "foot".to_owned(),
            ..HotCornerSettings::default()
        };
        let text = toml::to_string(&corners).expect("serialises");
        assert_eq!(
            toml::from_str::<HotCornerSettings>(&text).ok(),
            Some(corners)
        );
    }

    #[test]
    fn corner_actions_parse_their_slugs_and_match_serde() {
        for action in CornerAction::ALL {
            assert_eq!(
                CornerAction::parse(action.slug()),
                Some(*action),
                "{action:?}"
            );
            assert_eq!(
                serde_json::to_value(action).ok(),
                Some(serde_json::Value::from(action.slug())),
                "{action:?}: the stored word is the slug"
            );
        }
    }
}
