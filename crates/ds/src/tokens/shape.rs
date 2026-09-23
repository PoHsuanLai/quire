//! Radii (design/01-LAYOUT.md section 10). The first nine are the plan's names; the rest name
//! the literals design/04-COMPONENTS.md O-3 says the table must absorb.
//!
//! The names between the plan's are proposed (FINDINGS F13); the values are `S`'s.

use super::name::VarName;

/// One radius token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Radius {
    /// `--r-panel` 14: peek, command menu, hover card, editor.
    Panel,
    /// `--r-card` `12px 12px 12px 4px`: card and rows, the flap corner.
    Card,
    /// `--r-btn` 9: mini, button, tool.
    Btn,
    /// `--r-chip` `6px 6px 6px 2px`.
    Chip,
    /// `--r-pill` 999.
    Pill,
    /// `--r-field` 10: inputs, command pill, bubble.
    Field,
    /// `--r-menu` 12.
    Menu,
    /// `--r-item` 9: sidebar item, tooltip.
    Item,
    /// `--r-tile` 12: account tiles, editor field.
    Tile,
    /// `--r-window` 18.
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
    /// `--r-media` `10px 10px 10px 3px`: images, code blocks, attachments.
    Media,
}

impl Radius {
    /// Every radius, in stylesheet order.
    pub const ALL: [Radius; 17] = [
        Radius::Panel,
        Radius::Card,
        Radius::Btn,
        Radius::Chip,
        Radius::Pill,
        Radius::Field,
        Radius::Menu,
        Radius::Item,
        Radius::Tile,
        Radius::Window,
        Radius::MenuItem,
        Radius::BubbleButton,
        Radius::Small,
        Radius::Kbd,
        Radius::Tiny,
        Radius::Micro,
        Radius::Media,
    ];

    /// The custom property: `--r-panel`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            Radius::Panel => "--r-panel",
            Radius::Card => "--r-card",
            Radius::Btn => "--r-btn",
            Radius::Chip => "--r-chip",
            Radius::Pill => "--r-pill",
            Radius::Field => "--r-field",
            Radius::Menu => "--r-menu",
            Radius::Item => "--r-item",
            Radius::Tile => "--r-tile",
            Radius::Window => "--r-window",
            Radius::MenuItem => "--r-menu-item",
            Radius::BubbleButton => "--r-bubble-button",
            Radius::Small => "--r-small",
            Radius::Kbd => "--r-kbd",
            Radius::Tiny => "--r-tiny",
            Radius::Micro => "--r-micro",
            Radius::Media => "--r-media",
        })
    }

    /// The CSS value: `14px`, `12px 12px 12px 4px`.
    pub fn css(self) -> &'static str {
        match self {
            Radius::Panel => "14px",
            Radius::Card => "12px 12px 12px 4px",
            Radius::Btn => "9px",
            Radius::Chip => "6px 6px 6px 2px",
            Radius::Pill => "999px",
            Radius::Field => "10px",
            Radius::Menu => "12px",
            Radius::Item => "9px",
            Radius::Tile => "12px",
            Radius::Window => "18px",
            Radius::MenuItem => "8px",
            Radius::BubbleButton => "7px",
            Radius::Small => "6px",
            Radius::Kbd => "5px",
            Radius::Tiny => "4px",
            Radius::Micro => "3px",
            Radius::Media => "10px 10px 10px 3px",
        }
    }
}
