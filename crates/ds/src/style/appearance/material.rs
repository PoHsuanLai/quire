//! The eight materials (design/03-COLOR.md sections 17.1 and 17.3).

use super::blur::Blur;
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// What a surface is drawn in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Word)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    /// App windows: the Space gradient, its layers and grain, no blur.
    Window,
    /// The menu bar.
    Bar,
    /// The dock pill.
    Dock,
    /// Menus and popups off the bar and dock.
    Popover,
    /// The launcher panel, control center, notification center.
    Sheet,
    /// Notification banners.
    Toast,
    /// On-screen display (volume, brightness).
    Osd,
    /// Desktop widgets, the quick note.
    Widget,
}

impl Material {
    /// Whether the surface asks the compositor to blur behind it (design/03-COLOR.md section
    /// 17.2's Blur column: the window paints its own gradient, everything else is "behind").
    pub fn blur(self) -> Blur {
        match self {
            Material::Window => Blur::None,
            Material::Bar
            | Material::Dock
            | Material::Popover
            | Material::Sheet
            | Material::Toast
            | Material::Osd
            | Material::Widget => Blur::Behind,
        }
    }
}
