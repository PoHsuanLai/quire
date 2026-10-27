//! The settings of the machines in `ds-behaviour`: `switcher.*` (design/22 §3.11; design/13
//! §13.3.5) and `hot_corners.*` (design/13 §13.3.12). They live here, beside the schema, so the
//! shell and the compositor read one definition of each key; the keys stay in
//! `sill/settings.toml`. All Advanced, except the four hot corners, which one custom control draws.

use std::time::Duration;

use ds_behaviour::hot_corner::CornerParams;
use ds_behaviour::switcher::SwitcherParams;
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

use crate::schema::Page;
use crate::schema::corners::CornerBindings;
use crate::schema::corners_stored::StoredHotCorners;
use crate::units::{Ms, Px};

/// `switcher.*`: the app switcher's timing and cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "sill/settings.toml", domain = "switcher", page = Page::KeyboardAndShortcuts)]
#[non_exhaustive]
pub struct SwitcherSettings {
    #[settings(
        label = "Show delay",
        help = "How long to hold the keys before it appears.",
        section = "App switcher",
        range = "0..=500",
        unit = "ms",
        advanced
    )]
    pub show_delay_ms: Ms,
    #[settings(
        label = "Icon size",
        help = "How big app icons are.",
        section = "App switcher",
        range = "32..=256",
        unit = "px",
        advanced
    )]
    pub icon_size_px: Px,
    #[settings(
        label = "Cell size",
        help = "How big each app's cell is.",
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
        help = "How small icons shrink with many apps open.",
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
            icon_size_px: Px(64),
            cell_size_px: Px(80),
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

/// One of the desktop's own actions a hot corner can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
#[non_exhaustive]
pub enum CornerAction {
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
    /// The overview of the focused app's windows.
    AppWindows,
    /// Put the displays to sleep.
    DisplaySleep,
    /// Summon the assistant.
    Assistant,
}

/// `hot_corners.*`: what each screen corner does when the pointer rests in it, and the timing.
/// A corner is a list of bindings (an empty list leaves it inactive); a file that still holds the
/// older one word per corner (`top_left = "lock"`, with `top_left_command`) is read, and saving
/// writes the list form only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(try_from = "StoredHotCorners")]
#[settings(file = "sill/settings.toml", domain = "hot_corners", page = Page::Dock)]
#[non_exhaustive]
pub struct HotCornerSettings {
    #[settings(
        label = "Top left",
        help = "What happens when the pointer rests there.",
        section = "Hot Corners",
        text,
        control = "corners"
    )]
    pub top_left: CornerBindings,
    #[settings(
        label = "Top right",
        help = "What happens when the pointer rests there.",
        section = "Hot Corners",
        text,
        control = "corners"
    )]
    pub top_right: CornerBindings,
    #[settings(
        label = "Bottom left",
        help = "What happens when the pointer rests there.",
        section = "Hot Corners",
        text,
        control = "corners"
    )]
    pub bottom_left: CornerBindings,
    #[settings(
        label = "Bottom right",
        help = "What happens when the pointer rests there.",
        section = "Hot Corners",
        text,
        control = "corners"
    )]
    pub bottom_right: CornerBindings,
    #[settings(
        label = "Corner delay",
        help = "How long the pointer rests before it acts.",
        section = "Hot Corners",
        range = "0..=2000",
        unit = "ms",
        advanced
    )]
    pub dwell_ms: Ms,
    #[settings(
        label = "Corner re-arm time",
        help = "How long before a corner can act again.",
        section = "Hot Corners",
        range = "0..=5000",
        unit = "ms",
        advanced
    )]
    pub rearm_ms: Ms,
    #[settings(
        label = "Corner size",
        help = "How big the hidden hot spot is.",
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
            top_left: CornerBindings::default(),
            top_right: CornerBindings::default(),
            bottom_left: CornerBindings::default(),
            bottom_right: CornerBindings::default(),
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
