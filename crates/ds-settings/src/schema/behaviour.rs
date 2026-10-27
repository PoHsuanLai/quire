//! The setting of the hot-corner machine in `ds-behaviour`: `hot_corners.*` (design/13
//! §13.3.12). It lives here, beside the schema, so the shell and the compositor read one
//! definition of each key; the keys stay in `sill/settings.toml`. The four corners are one custom
//! control; the timing and the hot spot's size are constants of the machine
//! (`ds_behaviour::hot_corner::CornerParams`).

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

use crate::schema::Page;
use crate::schema::corners::CornerBindings;
use crate::schema::corners_stored::StoredHotCorners;

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

/// `hot_corners.*`: what each screen corner does when the pointer rests in it.
/// A corner is a list of bindings (an empty list leaves it inactive); a file that still holds the
/// older one word per corner (`top_left = "lock"`, with `top_left_command`) is read, and saving
/// writes the list form only.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, crate::SettingsSchema)]
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
}

#[cfg(test)]
mod tests {
    use ds_core::word::Word;

    use super::CornerAction;

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
