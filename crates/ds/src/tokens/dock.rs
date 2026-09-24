//! The dock's geometry tokens (the macOS polish pass, 2026-09-24; design/10-BEHAVIOUR-dock.md
//! sections 10.3.1 and 10.3.2): the tile, the gap between tiles, the pill's padding, the running
//! dot and the optional reflective floor, each a [`Tuned`] token its settings key moves. The
//! shell lays the dock out itself; these are the numbers it and quire's dock pieces
//! (`RunningDot`, `DockFloor`, an `IconView` plate) share.

use super::name::VarName;
use super::tuned::{Tuned, px};
use crate::geometry::Px;

const fn tuned(token: &'static str, input: &'static str, default: &'static str) -> Tuned {
    Tuned {
        token: VarName(token),
        input: VarName(input),
        default,
    }
}

/// `--dock-tile`: a tile at rest (`dock.tile_size_px`, 48; macOS `tilesize` 48, H).
pub const TILE: Tuned = tuned("--dock-tile", "--dock-tile-px", "48px");
/// `--dock-gap`: between two tiles (`dock.tile_gap_px`, 8).
pub const GAP: Tuned = tuned("--dock-gap", "--dock-gap-px", "8px");
/// `--dock-pad`: the pill's padding on every side (`dock.pill_padding_px`, 6).
pub const PAD: Tuned = tuned("--dock-pad", "--dock-pad-px", "6px");
/// `--dock-dot`: the running dot's diameter (`dock.running_dot_diameter_px`, 4).
pub const DOT: Tuned = tuned("--dock-dot", "--dock-dot-px", "4px");
/// `--dock-dot-gap`: from the tile's bottom edge to the dot's centre (`dock.running_dot_gap_px`,
/// 3), inside the pill's 6 px padding.
pub const DOT_GAP: Tuned = tuned("--dock-dot-gap", "--dock-dot-gap-px", "3px");
/// `--dock-floor`: the reflective floor's opacity, 0 or 1 (`dock.floor`, `Off`).
pub const FLOOR: Tuned = tuned("--dock-floor", "--dock-floor-on", "0");

/// Every dock token, in stylesheet order.
pub const DOCK_TOKENS: [Tuned; 6] = [TILE, GAP, PAD, DOT, DOT_GAP, FLOOR];

/// Whether the dock draws its reflective floor (`dock.floor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DockFloorSetting {
    /// No floor: the pill alone (the default).
    #[default]
    Off,
    /// A soft light band along the pill's floor, under the tiles.
    On,
}

/// The dock's geometry from the settings, written as the tokens' inputs on any element around
/// the dock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockMetrics {
    /// A tile at rest, 48.
    pub tile: Px,
    /// Between two tiles, 8.
    pub gap: Px,
    /// The pill's padding, 6.
    pub pad: Px,
    /// The running dot's diameter, 4.
    pub dot: Px,
    /// From the tile's bottom edge to the dot's centre, 3.
    pub dot_gap: Px,
    /// The reflective floor, off.
    pub floor: DockFloorSetting,
}

impl Default for DockMetrics {
    /// The keys' defaults (settled 2026-09-24, design/10 sections 10.3.1 and 10.3.2).
    fn default() -> Self {
        DockMetrics {
            tile: Px(48.0),
            gap: Px(8.0),
            pad: Px(6.0),
            dot: Px(4.0),
            dot_gap: Px(3.0),
            floor: DockFloorSetting::Off,
        }
    }
}

impl DockMetrics {
    /// Every input, inline: `--dock-tile-px:48px;…`.
    pub fn style_attr(&self) -> String {
        let length = |value: Px| px(value.0.round().clamp(0.0, 999.0) as u16);
        let floor = match self.floor {
            DockFloorSetting::Off => "0",
            DockFloorSetting::On => "1",
        };
        [
            TILE.write(&length(self.tile)),
            GAP.write(&length(self.gap)),
            PAD.write(&length(self.pad)),
            DOT.write(&length(self.dot)),
            DOT_GAP.write(&length(self.dot_gap)),
            FLOOR.write(floor),
        ]
        .concat()
    }

    /// The pill's height at rest: a tile and the padding above and below it (60 at the
    /// defaults).
    pub fn pill_height(&self) -> Px {
        Px(self.tile.0 + 2.0 * self.pad.0)
    }

    /// The pill's width at rest for `tiles` tiles: `n T + (n-1) g + 2 pad` (design/10 section
    /// 10.3.1).
    pub fn pill_width(&self, tiles: u16) -> Px {
        let n = f32::from(tiles);
        let gaps = f32::from(tiles.saturating_sub(1));
        Px(n * self.tile.0 + gaps * self.gap.0 + 2.0 * self.pad.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{DOCK_TOKENS, DockFloorSetting, DockMetrics};
    use crate::geometry::Px;

    #[test]
    fn the_defaults_write_what_the_stylesheet_falls_back_to() {
        let written = DockMetrics::default().style_attr();
        for token in DOCK_TOKENS {
            let want = token.write(token.default);
            assert!(written.contains(&want), "{want} not in {written}");
        }
    }

    #[test]
    fn the_pill_is_the_tiles_gaps_and_padding() {
        let dock = DockMetrics::default();
        assert_eq!(dock.pill_height(), Px(60.0));
        assert_eq!(dock.pill_width(3), Px(3.0 * 48.0 + 2.0 * 8.0 + 12.0));
        assert_eq!(dock.pill_width(1), Px(60.0));
        let on = DockMetrics {
            floor: DockFloorSetting::On,
            ..dock
        };
        assert!(on.style_attr().contains("--dock-floor-on:1;"));
    }
}
