//! The bar's status item geometry (design/22-SETTINGS.md section 3): the two custom
//! properties `IconButton { Status }` reads, written from the shell's settings.

use crate::core::geometry::units::Px;
use crate::style::tokens::name::VarName;

/// A bar's status item geometry, from the shell's settings (design/22-SETTINGS.md section 3,
/// `bar.status_icon_box_px`, `bar.status_glyph_px`, `bar.glyph_size_policy`): written as the
/// two custom properties `IconButton { Status }` reads, on any element around the items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusMetrics {
    /// The item's height (`bar.status_icon_box_px`, default 22); its slot is `--bar-status-w`
    /// (30) wide (design/29-SIZING.md).
    pub box_size: Px,
    /// The glyph inside it (`bar.status_glyph_px`, 16 by default; the box itself under the
    /// `IconSizeBar22` glyph policy).
    pub glyph: Px,
}

impl StatusMetrics {
    /// The property the box is read from.
    pub const BOX_VAR: VarName = VarName("--bar-status-box");
    /// The property the glyph is read from.
    pub const GLYPH_VAR: VarName = VarName("--bar-status-glyph");

    /// The inline declarations: `--bar-status-box:22px;--bar-status-glyph:16px;`.
    pub fn style_attr(&self) -> String {
        format!(
            "{}:{}px;{}:{}px;",
            Self::BOX_VAR.as_str(),
            self.box_size.0,
            Self::GLYPH_VAR.as_str(),
            self.glyph.0
        )
    }
}

impl Default for StatusMetrics {
    /// The keys' defaults: a 22 px box and a 16 px glyph.
    fn default() -> Self {
        StatusMetrics {
            box_size: Px(22.0),
            glyph: Px(16.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::geometry::units::Px;
    use crate::style::tokens::status::StatusMetrics;

    #[test]
    fn the_metrics_write_both_properties() {
        assert_eq!(
            StatusMetrics::default().style_attr(),
            "--bar-status-box:22px;--bar-status-glyph:16px;"
        );
        let full = StatusMetrics {
            box_size: Px(24.0),
            glyph: Px(24.0),
        };
        assert_eq!(
            full.style_attr(),
            "--bar-status-box:24px;--bar-status-glyph:24px;"
        );
    }
}
