//! The bar's and the text menus' geometry on the size ladder (design/29-SIZING.md sections 5.1,
//! 5.4 and 13): a 24 bar whose items are Regular controls (22), a status slot 30 wide with a 16
//! glyph, and a menu rounded 8 with its rows inset 5 (the reference's highlight is 5, not
//! concentric with the menu). The item heights, paddings and highlight a settings key can move
//! stay [`ShellMetrics`](super::ShellMetrics) tokens; these are the fixed ones.

use super::control_size::ControlSize;
use super::name::VarName;
use super::size_scale::WholePx;

/// The bar and text menus' fixed sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShellScale {
    /// The bar's height, 24.
    pub bar: WholePx,
    /// A status item's slot width, 30; its height is a Regular control's.
    pub status_width: WholePx,
    /// A text menu's radius, 8.
    pub menu_radius: WholePx,
    /// A text menu's inset round its rows, 5.
    pub menu_inset: WholePx,
}

/// The settled numbers (design/29 section 13 decision 2; the reference's menu).
pub const SHELL_SCALE: ShellScale = ShellScale {
    bar: WholePx(24),
    status_width: WholePx(30),
    menu_radius: WholePx(8),
    menu_inset: WholePx(5),
};

impl ShellScale {
    /// A bar item's height: a Regular control, 22.
    pub fn item(self) -> WholePx {
        ControlSize::Regular.scale().height
    }

    /// Every token and its value: `(--shell-bar-h, 24px)`, …
    pub fn tokens(self) -> [(VarName, String); 4] {
        [
            (VarName("--shell-bar-h"), self.bar.css()),
            (VarName("--bar-status-w"), self.status_width.css()),
            (VarName("--r-shell-menu"), self.menu_radius.css()),
            (VarName("--shell-menu-inset"), self.menu_inset.css()),
        ]
    }
}
