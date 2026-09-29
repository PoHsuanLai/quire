//! Radii (design/01-LAYOUT.md section 10). The first nine are the plan's names; the rest name
//! the literals design/04-COMPONENTS.md O-3 says the table must absorb.
//!
//! The names between the plan's are proposed; the values are `S`'s.

use crate::core::word::Word;
use crate::style::tokens::token::Token;

/// One radius token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "r-", kind = fixed)]
pub enum Radius {
    /// `--r-panel` 14: peek, command menu, hover card, editor.
    #[token(value = "14px")]
    Panel,
    /// `--r-card` `12px 12px 12px 4px`: card and rows, the flap corner.
    #[token(value = "12px 12px 12px 4px")]
    Card,
    /// `--r-btn` 9: mini, button, tool.
    #[token(value = "9px")]
    Btn,
    /// `--r-chip` `6px 6px 6px 2px`.
    #[token(value = "6px 6px 6px 2px")]
    Chip,
    /// `--r-pill` 999.
    #[token(value = "999px")]
    Pill,
    /// `--r-field` 10: inputs, command pill, bubble.
    #[token(value = "10px")]
    Field,
    /// `--r-menu` 12.
    #[token(value = "12px")]
    Menu,
    /// `--r-item` 9: sidebar item, tooltip.
    #[token(value = "9px")]
    Item,
    /// `--r-tile` 12: account tiles, editor field.
    #[token(value = "12px")]
    Tile,
    /// `--r-window` 18.
    #[token(value = "18px")]
    Window,
    /// `--r-menu-item` 8: menu item, prop row, foot button.
    #[token(value = "8px")]
    MenuItem,
    /// `--r-bubble-button` 7.
    #[token(value = "7px")]
    BubbleButton,
    /// `--r-small` 6: fly, gutter, quiet button.
    #[token(value = "6px")]
    Small,
    /// `--r-kbd` 5: key cap, favicon.
    #[token(value = "5px")]
    Kbd,
    /// `--r-tiny` 4: focus ring, provider mark.
    #[token(value = "4px")]
    Tiny,
    /// `--r-micro` 3: in-row provider mark.
    #[token(value = "3px")]
    Micro,
    /// `--r-media` `10px 10px 10px 3px`: images, code blocks, attachments.
    #[token(value = "10px 10px 10px 3px")]
    Media,
}

/// A material's corner, overriding the radius its recipe gives it (`--m-radius`): a radius
/// token, a length a settings key names (`dock.pill_radius_px`), or a
/// continuous-curvature squircle corner of that nominal radius (the macOS polish pass).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    /// One of the radius tokens.
    Token(Radius),
    /// A length in logical pixels.
    Px(crate::core::geometry::units::Px),
    /// A squircle corner (design/08-ICONS.md section 2.1's `n = 5` superellipse) of this nominal
    /// radius: it reaches `2 r` along each edge and, at 45 degrees, sits nearer the box's corner
    /// than a circle of radius `r` does. Drawn as a `mask-image` (`data-corner="squircle"`);
    /// the element's shadows and hairline follow the circle that touches it at 45 degrees.
    Squircle(crate::core::geometry::units::Px),
}

impl Corner {
    /// The CSS radius: `var(--r-panel)`, `22px`; for a squircle, the circle its shadows follow.
    pub fn css(self) -> String {
        match self {
            Corner::Token(radius) => radius.var().reference(),
            Corner::Px(length) => format!("{}px", length.0),
            Corner::Squircle(length) => format!("{}px", squircle_shadow_radius(length)),
        }
    }

    /// The `data-corner` word, for a corner the stylesheet draws itself: `squircle`.
    pub fn attribute(self) -> Option<&'static str> {
        match self {
            Corner::Squircle(_) => Some("squircle"),
            Corner::Token(_) | Corner::Px(_) => None,
        }
    }

    /// The inline declarations a squircle corner needs besides `--m-radius`: its extent,
    /// `--sq-k:44px;`, and the radius its shadows follow, `--r-squircle:19.45px;`.
    pub fn squircle_style(self) -> String {
        match self {
            Corner::Squircle(length) => {
                let extent = f64::from(length.0) * crate::style::tokens::plate::EXTENT_PER_RADIUS;
                format!(
                    "--sq-k:{}px;--r-squircle:{}px;",
                    round2(extent),
                    squircle_shadow_radius(length)
                )
            }
            Corner::Token(_) | Corner::Px(_) => String::new(),
        }
    }
}

/// The circle a squircle of nominal radius `length` is inscribed in at 45 degrees.
fn squircle_shadow_radius(length: crate::core::geometry::units::Px) -> f64 {
    round2(f64::from(length.0) * crate::style::tokens::plate::shadow_radius_share())
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

impl From<Radius> for Corner {
    fn from(radius: Radius) -> Self {
        Corner::Token(radius)
    }
}
