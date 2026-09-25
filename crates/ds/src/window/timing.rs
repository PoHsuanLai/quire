//! The frame's three proposed numbers, each a settings key (design/22-SETTINGS.md section 3.11;
//! design/13-BEHAVIOUR-menus-windows.md section 13.3.11). The caller reads them from its settings
//! and passes them in; the default is each key's default.

use crate::geometry::Px;
use std::time::Duration;

/// When the titlebar starts a move and when the green light opens the tiling menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTiming {
    /// How far a press on the titlebar travels, on either axis, before it becomes a move
    /// (`window.move_threshold_px`, default 4): less is a click, so a double-click still zooms.
    pub move_threshold: Px,
    /// How long the green light is held before the tiling menu opens
    /// (`window.tile_menu_press_ms`, default 500).
    pub menu_press: Duration,
    /// How long the pointer rests on the green light before the tiling menu opens
    /// (`window.tile_menu_hover_ms`, default 800: the 450 ms hover intent and a further 350 ms,
    /// so passing over the light on the way to the close button does not open it).
    pub menu_hover: Duration,
}

impl Default for FrameTiming {
    /// The keys' defaults.
    fn default() -> Self {
        FrameTiming {
            move_threshold: Px(4.0),
            menu_press: Duration::from_millis(500),
            menu_hover: Duration::from_millis(800),
        }
    }
}
