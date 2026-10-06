//! Radii (design/30-CATALOGUE.md section 3.2). The first nine are the plan's names
//! (design/01-LAYOUT.md section 10); the rest name the literals design/04-COMPONENTS.md O-3 says
//! the table must absorb.

use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One radius token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "r-", kind = fixed, css = radius_css)]
pub enum Radius {
    /// `--r-panel` 12: sheet, alert, peek, command menu, hover card, editor.
    Panel,
    /// `--r-card` 12: card and rows.
    Card,
    /// `--r-btn` 6: mini, button, tool.
    Btn,
    /// `--r-chip` 6.
    Chip,
    /// `--r-pill` 999.
    Pill,
    /// `--r-field` 8: inputs, command pill, bubble.
    Field,
    /// `--r-menu` 10.
    Menu,
    /// `--r-item` 6: sidebar item, tooltip.
    Item,
    /// `--r-tile` 12: account tiles, editor field.
    Tile,
    /// `--r-window` 10.
    Window,
    /// `--r-menu-item` 6: menu item, prop row, foot button.
    MenuItem,
    /// `--r-small` 6: fly, gutter, quiet button.
    Small,
    /// `--r-kbd` 5: key cap, favicon.
    Kbd,
    /// `--r-tiny` 5: focus ring, provider mark.
    Tiny,
    /// `--r-micro` 4: in-row provider mark.
    Micro,
    /// `--r-media` 12: images, code blocks, attachments.
    Media,
    /// `--r-icon-tile` 6: the 24 px icon tile leading a grouped-list row.
    IconTile,
    /// `--r-group` 12: an inset grouped list or form group (the card's radius under its own
    /// name, so a group can move without moving every card).
    Group,
}

impl Radius {
    /// The radius as CSS. The Look (design/34-MODERN-LOOK.md section 2.1, step 1): control 6,
    /// field 8, menu 10, popover, card, group and panel 12; the window stays the host's 10.
    pub fn value(self) -> &'static str {
        match self {
            Radius::Panel | Radius::Card | Radius::Tile | Radius::Media | Radius::Group => "12px",
            Radius::Window | Radius::Menu => "10px",
            Radius::Field => "8px",
            Radius::Btn | Radius::MenuItem | Radius::IconTile => "6px",
            Radius::Chip | Radius::Item | Radius::Small => "6px",
            Radius::Kbd | Radius::Tiny => "5px",
            Radius::Pill => "999px",
            Radius::Micro => "4px",
        }
    }
}

/// The smallest radius a concentric inner shape takes (design/34-MODERN-LOOK.md section 2.1).
pub const CONCENTRIC_FLOOR: ds_core::geometry::units::Px = ds_core::geometry::units::Px(4.0);

/// The concentric rule (design/34 section 2.1, WWDC25 "concentric shapes"): the radius of a
/// shape inset `inset` inside an outer corner of radius `outer` is the outer radius less the
/// inset, never below [`CONCENTRIC_FLOOR`]. A capsule's radius is half its height and never goes
/// through here.
pub fn concentric(
    outer: ds_core::geometry::units::Px,
    inset: ds_core::geometry::units::Px,
) -> ds_core::geometry::units::Px {
    ds_core::geometry::units::Px((outer.0 - inset.0).max(CONCENTRIC_FLOOR.0))
}

/// A radius token as the stylesheet writes it.
fn radius_css(token: Radius, _: TokenScope) -> CssValue {
    CssValue::fixed(token.value())
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
                let extent = f64::from(length.0) * crate::tokens::plate::EXTENT_PER_RADIUS;
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
    round2(f64::from(length.0) * crate::tokens::plate::shadow_radius_share())
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

impl From<Radius> for Corner {
    fn from(radius: Radius) -> Self {
        Corner::Token(radius)
    }
}

#[cfg(test)]
mod tests {
    use super::{CONCENTRIC_FLOOR, Radius, concentric};
    use ds_core::geometry::units::Px;

    #[test]
    fn the_step_one_radii_are_design_34_section_2_1() {
        let value = Radius::value;
        assert_eq!(value(Radius::Panel), "12px");
        assert_eq!(value(Radius::Card), "12px");
        assert_eq!(value(Radius::Group), "12px");
        assert_eq!(value(Radius::Btn), "6px");
        assert_eq!(value(Radius::Field), "8px");
        assert_eq!(value(Radius::Menu), "10px");
        assert_eq!(value(Radius::MenuItem), "6px");
        assert_eq!(value(Radius::IconTile), "6px");
        assert_eq!(value(Radius::Tiny), "5px");
        assert_eq!(value(Radius::Micro), "4px");
        assert_eq!(value(Radius::Window), "10px");
    }

    #[test]
    fn a_group_follows_the_card_and_a_menu_item_sits_inside_its_menu() {
        assert_eq!(Radius::Group.value(), Radius::Card.value());
        let item: f32 = Radius::MenuItem
            .value()
            .trim_end_matches("px")
            .parse()
            .unwrap_or(f32::NAN);
        assert!(item <= concentric(Px(10.0), Px(4.0)).0, "{item}");
    }

    #[test]
    fn the_concentric_radius_is_outer_less_inset_floored() {
        assert_eq!(concentric(Px(26.0), Px(8.0)), Px(18.0));
        assert_eq!(concentric(Px(26.0), Px(12.0)), Px(14.0));
        assert_eq!(concentric(Px(10.0), Px(8.0)), CONCENTRIC_FLOOR);
    }
}
