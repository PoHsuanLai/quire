//! The window chrome's fixed geometry (design/04-COMPONENTS.md "Window frame"; design/13
//! section 13.3.11): the numbers the window frame's stylesheet reads as `--chrome-*` and a
//! compositor that draws the chrome itself reads as lengths, so the two cannot drift.

use crate::tokens::size_scale::WholePx;

/// The titlebar, the lights and the resize edges, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChromeScale {
    /// The titlebar's height.
    pub titlebar_height: WholePx,
    /// A light's diameter.
    pub light_size: WholePx,
    /// The space between two lights.
    pub light_gap: WholePx,
    /// How far the first light is from the window's left edge; the titlebar pads its right end
    /// by the same.
    pub light_inset: WholePx,
    /// How far the centred title keeps from each side, so it clears the lights.
    pub title_inset: WholePx,
    /// The thickness of a side's resize zone.
    pub edge: WholePx,
    /// The side of a corner's square resize zone, which a side's zone stops short of.
    pub edge_corner: WholePx,
}

/// The settled numbers (the macOS window).
pub const CHROME_SCALE: ChromeScale = ChromeScale {
    titlebar_height: WholePx(28),
    light_size: WholePx(12),
    light_gap: WholePx(8),
    light_inset: WholePx(13),
    title_inset: WholePx(84),
    edge: WholePx(4),
    edge_corner: WholePx(12),
};
