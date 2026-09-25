//! The OSD card's margin token (sill FINDINGS Q76; design/22-SETTINGS.md section 3.16
//! `osd.margin_px`): the gap from the bar's reserve to the card at the top right, or from the
//! dock's at the bottom centre. A [`Tuned`] token so the setting reaches the card through one
//! inline write ([`OsdMetrics::style_attr`]) on any element around it, as the dock's geometry.

use super::name::VarName;
use super::tuned::{Tuned, px};
use crate::geometry::Px;

/// `--osd-margin`: the card's gap from the edge it is anchored to (`osd.margin_px`, 24).
pub const MARGIN: Tuned = Tuned {
    token: VarName("--osd-margin"),
    input: VarName("--osd-margin-px"),
    default: "24px",
};

/// Every OSD token, in stylesheet order.
pub const OSD_TOKENS: [Tuned; 1] = [MARGIN];

/// The OSD's geometry from the settings, written as the tokens' inputs on any element around the
/// card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OsdMetrics {
    /// The card's gap from the bar's reserve (top right) or the dock's (bottom centre), 24.
    pub margin: Px,
}

impl Default for OsdMetrics {
    /// The key's default (design/22 section 3.16, proposed 2026-09-25).
    fn default() -> Self {
        OsdMetrics { margin: Px(24.0) }
    }
}

impl OsdMetrics {
    /// Every input, inline: `--osd-margin-px:24px;`. A margin outside the key's `0..=400` is
    /// held to it.
    pub fn style_attr(&self) -> String {
        MARGIN.write(&px(self.margin.0.round().clamp(0.0, 400.0) as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::{OSD_TOKENS, OsdMetrics};
    use crate::geometry::Px;

    #[test]
    fn the_default_writes_what_the_stylesheet_falls_back_to() {
        let written = OsdMetrics::default().style_attr();
        for token in OSD_TOKENS {
            assert_eq!(written, token.write(token.default));
        }
    }

    #[test]
    fn the_margin_is_held_to_the_keys_range() {
        let far = OsdMetrics { margin: Px(900.0) };
        assert_eq!(far.style_attr(), "--osd-margin-px:400px;");
        let near = OsdMetrics { margin: Px(-3.0) };
        assert_eq!(near.style_attr(), "--osd-margin-px:0px;");
    }
}
