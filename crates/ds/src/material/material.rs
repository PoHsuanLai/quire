//! The eight materials (design/03-COLOR.md sections 17.1 and 17.3).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::blur::Blur;
use serde::{Deserialize, Serialize};

/// What a surface is drawn in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
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
    /// Every material, in the gallery's order.
    pub const ALL: [Material; 8] = [
        Material::Window,
        Material::Bar,
        Material::Dock,
        Material::Popover,
        Material::Sheet,
        Material::Toast,
        Material::Osd,
        Material::Widget,
    ];

    /// The `data-material` value.
    pub fn slug(self) -> &'static str {
        todo!()
    }

    /// Whether the surface asks the compositor to blur behind it.
    pub fn blur(self) -> Blur {
        todo!()
    }
}
