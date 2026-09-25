//! The widget grid's unit (sill FINDINGS Q182; design/22-SETTINGS.md section 3.20,
//! design/20-SURFACES.md section 1.14): a cell's side and the gap between cells, each a
//! [`Tuned`] token its `widgets.*` key moves through one inline write
//! ([`WidgetMetrics::style_attr`]) on any element around the widgets. A `WidgetFrame` sizes
//! itself from both, so the setting reaches the card and not only sill's layout.

use super::name::VarName;
use super::tuned::{Tuned, px};
use crate::geometry::Px;

/// `--widget-cell` (`widgets.desktop_cell_px`, 164): a small widget's side.
pub const CELL: Tuned = Tuned {
    token: VarName("--widget-cell"),
    input: VarName("--widget-cell-px"),
    default: "164px",
};

/// `--widget-gap` (`widgets.desktop_gap_px`, 16): between two cells, so a medium widget is two
/// cells and one gap wide.
pub const GAP: Tuned = Tuned {
    token: VarName("--widget-gap"),
    input: VarName("--widget-gap-px"),
    default: "16px",
};

/// Every widget token, in stylesheet order.
pub const WIDGET_TOKENS: [Tuned; 2] = [CELL, GAP];

/// The widget grid from the settings, written as the tokens' inputs on any element around the
/// widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidgetMetrics {
    /// A cell's side, 164 (the key's range is 120 to 240).
    pub cell: Px,
    /// Between cells, 16 (the key's range is 0 to 48).
    pub gap: Px,
}

impl Default for WidgetMetrics {
    /// The keys' defaults (design/22 section 3.20, proposed 2026-09-26).
    fn default() -> Self {
        WidgetMetrics {
            cell: Px(164.0),
            gap: Px(16.0),
        }
    }
}

impl WidgetMetrics {
    /// Both inputs, inline: `--widget-cell-px:164px;--widget-gap-px:16px;`, each held to its
    /// key's range so a hand-edited file cannot draw a widget smaller than its content.
    pub fn style_attr(&self) -> String {
        let held = |value: Px, low: f32, high: f32| px(value.0.round().clamp(low, high) as u16);
        [
            CELL.write(&held(self.cell, 120.0, 240.0)),
            GAP.write(&held(self.gap, 0.0, 48.0)),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::{WIDGET_TOKENS, WidgetMetrics};
    use crate::geometry::Px;

    #[test]
    fn the_defaults_write_what_the_stylesheet_falls_back_to() {
        let written = WidgetMetrics::default().style_attr();
        let want: String = WIDGET_TOKENS
            .iter()
            .map(|token| token.write(token.default))
            .collect();
        assert_eq!(written, want);
    }

    #[test]
    fn each_length_is_held_to_its_keys_range() {
        let wild = WidgetMetrics {
            cell: Px(400.0),
            gap: Px(-5.0),
        };
        assert_eq!(
            wild.style_attr(),
            "--widget-cell-px:240px;--widget-gap-px:0px;"
        );
        let small = WidgetMetrics {
            cell: Px(80.0),
            gap: Px(90.0),
        };
        assert_eq!(
            small.style_attr(),
            "--widget-cell-px:120px;--widget-gap-px:48px;"
        );
    }
}
