//! The shell type scale (the macOS polish pass, 2026-09-24): the bar's items, text menus, the
//! launcher and tooltips, each size a [`Tuned`] token a settings key can move. The defaults are
//! the macOS numbers design/13-BEHAVIOUR-menus-windows.md section 13.2 records (menu bar text
//! 13 pt, menu rows about 22 pt, 13 pt menu text) and the launcher's Spotlight-like scale
//! (design/13 section 13.3.9); `IconButton { Status }`, `MenuBarItem`, `Menu`, `CommandPalette`
//! in a surface and `Tooltip` read them, so a consumer gets them with no prop.

use super::name::VarName;
use super::tuned::{Tuned, px};
use crate::geometry::Px;

/// A CSS font weight: 400, 500, 700.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

const fn tuned(token: &'static str, input: &'static str, default: &'static str) -> Tuned {
    Tuned {
        token: VarName(token),
        input: VarName(input),
        default,
    }
}

/// `--fs-shell-bar`: a bar item's text (`bar.item_font_px`, 13).
pub const BAR_FONT: Tuned = tuned("--fs-shell-bar", "--shell-bar-font", "13px");
/// `--fw-shell-bar`: its weight (`bar.item_font_weight`, 500).
pub const BAR_WEIGHT: Tuned = tuned("--fw-shell-bar", "--shell-bar-weight", "500");
/// `--shell-bar-item`: the hover and open pill's height (`bar.open_title_pill_height_px`, 24).
pub const BAR_ITEM: Tuned = tuned("--shell-bar-item", "--shell-bar-item-h", "24px");
/// `--shell-bar-pad`: the pill's side padding around a text item (`bar.title_padding_px`, 10).
pub const BAR_PAD: Tuned = tuned("--shell-bar-pad", "--shell-bar-pad-x", "10px");
/// `--r-shell-bar-item`: the pill's radius (`bar.item_radius_px`, 4).
pub const BAR_RADIUS: Tuned = tuned("--r-shell-bar-item", "--shell-bar-radius", "4px");
/// `--fs-shell-menu`: a text menu's items (`menus.font_px`, 13).
pub const MENU_FONT: Tuned = tuned("--fs-shell-menu", "--shell-menu-font", "13px");
/// `--shell-menu-row`: a text menu row's height (`menus.item_height_px`, 22).
pub const MENU_ROW: Tuned = tuned("--shell-menu-row", "--shell-menu-row-h", "22px");
/// `--shell-menu-sep`: the margin above and below a separator (`menus.separator_margin_px`, 5).
pub const MENU_SEPARATOR: Tuned = tuned("--shell-menu-sep", "--shell-menu-sep-m", "5px");
/// `--r-shell-highlight`: the selected row's inset highlight (`menus.highlight_radius_px`, 6).
pub const HIGHLIGHT_RADIUS: Tuned = tuned("--r-shell-highlight", "--shell-highlight-radius", "6px");
/// `--fs-shell-field`: the launcher's query (`launcher.field_font_px`, 22).
pub const FIELD_FONT: Tuned = tuned("--fs-shell-field", "--shell-field-font", "22px");
/// `--fw-shell-field`: its weight (`launcher.field_font_weight`, 500).
pub const FIELD_WEIGHT: Tuned = tuned("--fw-shell-field", "--shell-field-weight", "500");
/// `--shell-field-glyph`: the search glyph beside it (`launcher.field_glyph_px`, 20).
pub const FIELD_GLYPH: Tuned = tuned("--shell-field-glyph", "--shell-field-glyph-px", "20px");
/// `--fs-shell-row`: a launcher row's title (`launcher.row_title_px`, 14).
pub const ROW_TITLE: Tuned = tuned("--fs-shell-row", "--shell-row-font", "14px");
/// `--fs-shell-detail`: a launcher row's detail (`launcher.row_detail_px`, 12).
pub const ROW_DETAIL: Tuned = tuned("--fs-shell-detail", "--shell-detail-font", "12px");
/// `--fs-shell-tip`: a tooltip's label (`menus.tooltip_font_px`, 12).
pub const TIP_FONT: Tuned = tuned("--fs-shell-tip", "--shell-tip-font", "12px");

/// Every shell token, in stylesheet order.
pub const SHELL_TOKENS: [Tuned; 15] = [
    BAR_FONT,
    BAR_WEIGHT,
    BAR_ITEM,
    BAR_PAD,
    BAR_RADIUS,
    MENU_FONT,
    MENU_ROW,
    MENU_SEPARATOR,
    HIGHLIGHT_RADIUS,
    FIELD_FONT,
    FIELD_WEIGHT,
    FIELD_GLYPH,
    ROW_TITLE,
    ROW_DETAIL,
    TIP_FONT,
];

/// The bar's text items and their pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarType {
    /// Text size, 13.
    pub font: Px,
    /// Text weight, 500.
    pub weight: FontWeight,
    /// The hover and open pill's height, 24.
    pub item_height: Px,
    /// The pill's side padding around a text item, 10.
    pub item_padding: Px,
    /// The pill's radius, 4.
    pub item_radius: Px,
}

/// Text menus: the bar's, context and dock menus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuType {
    /// Item text, 13.
    pub font: Px,
    /// Row height, 22.
    pub row: Px,
    /// Separator margin above and below, 5.
    pub separator_margin: Px,
    /// The selected row's highlight radius, 6.
    pub highlight_radius: Px,
}

/// The launcher: its field and rows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LauncherType {
    /// The query's size, 22.
    pub field_font: Px,
    /// The query's weight, 500.
    pub field_weight: FontWeight,
    /// The search glyph, 20.
    pub field_glyph: Px,
    /// A row's title, 14.
    pub row_title: Px,
    /// A row's detail, 12.
    pub row_detail: Px,
}

/// The shell type scale from the settings, written as the tuned tokens' inputs on any element
/// around the surfaces (the bar's root container, a popup's, the launcher's).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellMetrics {
    /// The bar.
    pub bar: BarType,
    /// Text menus.
    pub menu: MenuType,
    /// The launcher.
    pub launcher: LauncherType,
    /// A tooltip's label, 12.
    pub tip: Px,
}

impl Default for ShellMetrics {
    /// The keys' defaults (settled 2026-09-24, design/13 sections 13.3.1, 13.3.3, 13.3.9).
    fn default() -> Self {
        ShellMetrics {
            bar: BarType {
                font: Px(13.0),
                weight: FontWeight(500),
                item_height: Px(24.0),
                item_padding: Px(10.0),
                item_radius: Px(4.0),
            },
            menu: MenuType {
                font: Px(13.0),
                row: Px(22.0),
                separator_margin: Px(5.0),
                highlight_radius: Px(6.0),
            },
            launcher: LauncherType {
                field_font: Px(22.0),
                field_weight: FontWeight(500),
                field_glyph: Px(20.0),
                row_title: Px(14.0),
                row_detail: Px(12.0),
            },
            tip: Px(12.0),
        }
    }
}

impl ShellMetrics {
    /// Every input, inline: `--shell-bar-font:13px;…`.
    pub fn style_attr(&self) -> String {
        let length = |value: Px| px(value.0.round().clamp(0.0, 999.0) as u16);
        let weight = |value: FontWeight| value.0.to_string();
        let ShellMetrics {
            bar,
            menu,
            launcher,
            tip,
        } = *self;
        [
            BAR_FONT.write(&length(bar.font)),
            BAR_WEIGHT.write(&weight(bar.weight)),
            BAR_ITEM.write(&length(bar.item_height)),
            BAR_PAD.write(&length(bar.item_padding)),
            BAR_RADIUS.write(&length(bar.item_radius)),
            MENU_FONT.write(&length(menu.font)),
            MENU_ROW.write(&length(menu.row)),
            MENU_SEPARATOR.write(&length(menu.separator_margin)),
            HIGHLIGHT_RADIUS.write(&length(menu.highlight_radius)),
            FIELD_FONT.write(&length(launcher.field_font)),
            FIELD_WEIGHT.write(&weight(launcher.field_weight)),
            FIELD_GLYPH.write(&length(launcher.field_glyph)),
            ROW_TITLE.write(&length(launcher.row_title)),
            ROW_DETAIL.write(&length(launcher.row_detail)),
            TIP_FONT.write(&length(tip)),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::{SHELL_TOKENS, ShellMetrics};

    #[test]
    fn the_defaults_write_what_the_stylesheet_falls_back_to() {
        let written = ShellMetrics::default().style_attr();
        for token in SHELL_TOKENS {
            let want = token.write(token.default);
            assert!(written.contains(&want), "{want} not in {written}");
        }
    }

    #[test]
    fn a_changed_key_moves_only_its_input() {
        let mut metrics = ShellMetrics::default();
        metrics.menu.row = crate::geometry::Px(24.0);
        let written = metrics.style_attr();
        assert!(written.contains("--shell-menu-row-h:24px;"));
        assert!(written.contains("--shell-menu-font:13px;"));
    }
}
