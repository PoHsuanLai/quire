//! Radii, per Look (design/30-CATALOGUE.md section 3.2). The first nine are the plan's names
//! (design/01-LAYOUT.md section 10); the rest name the literals design/04-COMPONENTS.md O-3 says
//! the table must absorb.

use crate::style::look::Look;
use crate::style::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One radius token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "r-", kind = fixed, css = radius_css)]
pub enum Radius {
    /// `--r-panel` 10: peek, command menu, hover card, editor.
    Panel,
    /// `--r-card` 10: card and rows.
    Card,
    /// `--r-btn` 5: mini, button, tool.
    Btn,
    /// `--r-chip` 6.
    Chip,
    /// `--r-pill` 999.
    Pill,
    /// `--r-field` 5: inputs, command pill, bubble.
    Field,
    /// `--r-menu` 8.
    Menu,
    /// `--r-item` 6: sidebar item, tooltip.
    Item,
    /// `--r-tile` 10: account tiles, editor field.
    Tile,
    /// `--r-window` 10.
    Window,
    /// `--r-menu-item` 8: menu item, prop row, foot button.
    MenuItem,
    /// `--r-bubble-button` 7.
    BubbleButton,
    /// `--r-small` 6: fly, gutter, quiet button.
    Small,
    /// `--r-kbd` 5: key cap, favicon.
    Kbd,
    /// `--r-tiny` 4: focus ring, provider mark.
    Tiny,
    /// `--r-micro` 3: in-row provider mark.
    Micro,
    /// `--r-media` 10: images, code blocks, attachments.
    Media,
}

impl Radius {
    /// The radius in `look`, as CSS. The Mac Look (design/30-CATALOGUE.md section 3.2) has
    /// small, uniform radii: control and field 5, menu 8, popover, card and window 10.
    pub fn value(self, look: Look) -> &'static str {
        match look {
            Look::Mac => match self {
                Radius::Panel | Radius::Card | Radius::Tile | Radius::Window | Radius::Media => {
                    "10px"
                }
                Radius::Btn | Radius::Field | Radius::Kbd => "5px",
                Radius::Chip | Radius::Item | Radius::Small => "6px",
                Radius::Pill => "999px",
                Radius::Menu | Radius::MenuItem => "8px",
                Radius::BubbleButton => "7px",
                Radius::Tiny => "4px",
                Radius::Micro => "3px",
            },
        }
    }
}

/// A radius token as the stylesheet writes it, in the scope's Look.
fn radius_css(token: Radius, scope: TokenScope) -> CssValue {
    CssValue::fixed(token.value(scope.look))
}

/// A material's corner, overriding the radius its recipe gives it (`--m-radius`): a radius
/// token, a length a settings key names (`dock.pill_radius_px`), or a
/// continuous-curvature squircle corner of that nominal radius (the macOS polish pass).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    /// One of the radius tokens.
    Token(Radius),
    /// A length in logical pixels.
    Px(ds_core::geometry::units::Px),
    /// A squircle corner (design/08-ICONS.md section 2.1's `n = 5` superellipse) of this nominal
    /// radius: it reaches `2 r` along each edge and, at 45 degrees, sits nearer the box's corner
    /// than a circle of radius `r` does. Drawn as a `mask-image` (`data-corner="squircle"`);
    /// the element's shadows and hairline follow the circle that touches it at 45 degrees.
    Squircle(ds_core::geometry::units::Px),
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
fn squircle_shadow_radius(length: ds_core::geometry::units::Px) -> f64 {
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
