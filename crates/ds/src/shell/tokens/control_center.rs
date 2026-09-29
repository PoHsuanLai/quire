//! The control center's geometry on the size ladder (design/29-SIZING.md sections 7 and 13): a
//! 320 panel, 10 of padding and 10 between modules, modules rounded 8 inside a panel rounded 18
//! (8 + 10, R6), a toggle tile two Large controls tall, and a level module that is its padding,
//! a 16 header, a 6 gap and a Regular capsule. sill sizes its popup from these (R9) rather than
//! from its own copies.

use super::control_size::ControlSize;
use super::name::VarName;
use super::size_scale::WholePx;

/// The control center's sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlCenterScale {
    /// The panel's width, 320.
    pub width: WholePx,
    /// Inside the panel's edge and inside a module, 10.
    pub padding: WholePx,
    /// Between modules, 10.
    pub gap: WholePx,
    /// A module's radius, 8.
    pub module_radius: WholePx,
    /// A toggle tile's height, 56.
    pub tile: WholePx,
    /// A module header's line, 16.
    pub head: WholePx,
    /// Between a module's header and its control, 6.
    pub head_gap: WholePx,
}

/// The settled numbers (design/29 section 13).
pub const CONTROL_CENTER: ControlCenterScale = ControlCenterScale {
    width: WholePx(320),
    padding: WholePx(10),
    gap: WholePx(10),
    module_radius: WholePx(8),
    tile: WholePx(56),
    head: WholePx(16),
    head_gap: WholePx(6),
};

impl ControlCenterScale {
    /// The panel's radius: a module's plus the padding round it (R6), 18.
    pub fn panel_radius(self) -> WholePx {
        WholePx(self.module_radius.0 + self.padding.0)
    }

    /// A level module's height: padding, header, gap, a Regular capsule, padding; 64.
    pub fn level_module(self) -> WholePx {
        WholePx(
            2 * self.padding.0
                + self.head.0
                + self.head_gap.0
                + ControlSize::Regular.scale().height.0,
        )
    }

    /// Every token and its value: `(--cc-width, 320px)`, …
    pub fn tokens(self) -> [(VarName, String); 9] {
        [
            (VarName("--cc-width"), self.width.css()),
            (VarName("--cc-pad"), self.padding.css()),
            (VarName("--cc-gap"), self.gap.css()),
            (VarName("--cc-module-r"), self.module_radius.css()),
            (VarName("--cc-panel-r"), self.panel_radius().css()),
            (VarName("--cc-tile-h"), self.tile.css()),
            (VarName("--cc-head"), self.head.css()),
            (VarName("--cc-head-gap"), self.head_gap.css()),
            (VarName("--cc-level-h"), self.level_module().css()),
        ]
    }
}
