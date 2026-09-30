//! The shell type scale (the macOS polish pass, 2026-09-24): the bar's items, text menus, the
//! launcher and tooltips, each size a tuned token a settings key can move. The defaults are
//! the macOS numbers design/13-BEHAVIOUR-menus-windows.md section 13.2 records (menu bar text
//! 13 pt, menu rows about 22 pt, 13 pt menu text) and the launcher's Spotlight-like scale
//! (design/13 section 13.3.9); `IconButton { Status }`, `MenuBarItem`, `Menu`, `CommandPalette`
//! in a surface and `Tooltip` read them, so a consumer gets them with no prop.

use crate::style::tokens::token::Token;
use crate::style::tokens::tuned::px;
use ds_core::geometry::units::Px;
use ds_core::word::Word;

/// One token of the shell type scale, each a tuned token a settings key can move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = tuned)]
pub enum ShellType {
    /// `--fs-shell-bar`: a bar item's text (`bar.item_font_px`, 13).
    #[token(name = "fs-shell-bar", input = "--shell-bar-font", value = "13px")]
    BarFont,
    /// `--fw-shell-bar`: its weight (`bar.item_font_weight`, 500).
    #[token(name = "fw-shell-bar", input = "--shell-bar-weight", value = "500")]
    BarWeight,
    /// `--shell-bar-item`: the hover and open pill's height (`bar.open_title_pill_height_px`, 22: a
    /// Regular control, design/29-SIZING.md).
    #[token(name = "shell-bar-item", input = "--shell-bar-item-h", value = "22px")]
    BarItem,
    /// `--shell-bar-pad`: the pill's side padding around a text item (`bar.title_padding_px`, 8).
    #[token(name = "shell-bar-pad", input = "--shell-bar-pad-x", value = "8px")]
    BarPad,
    /// `--r-shell-bar-item`: the pill's radius (`bar.item_radius_px`, 5: a Regular control's).
    #[token(name = "r-shell-bar-item", input = "--shell-bar-radius", value = "5px")]
    BarRadius,
    /// `--fs-shell-menu`: a text menu's items (`menus.font_px`, 13).
    #[token(name = "fs-shell-menu", input = "--shell-menu-font", value = "13px")]
    MenuFont,
    /// `--shell-menu-row`: a text menu row's height (`menus.item_height_px`, 22).
    #[token(name = "shell-menu-row", input = "--shell-menu-row-h", value = "22px")]
    MenuRow,
    /// `--shell-menu-sep`: the margin above and below a separator (`menus.separator_margin_px`, 5).
    #[token(name = "shell-menu-sep", input = "--shell-menu-sep-m", value = "5px")]
    MenuSeparator,
    /// `--r-shell-highlight`: the selected row's inset highlight (`menus.highlight_radius_px`, 5: the
    /// reference's, design/29-SIZING.md section 5.4).
    #[token(
        name = "r-shell-highlight",
        input = "--shell-highlight-radius",
        value = "5px"
    )]
    HighlightRadius,
    /// `--fs-shell-field`: the launcher's query (`launcher.field_font_px`, 22).
    #[token(name = "fs-shell-field", input = "--shell-field-font", value = "22px")]
    FieldFont,
    /// `--fw-shell-field`: its weight (`launcher.field_font_weight`, 500).
    #[token(name = "fw-shell-field", input = "--shell-field-weight", value = "500")]
    FieldWeight,
    /// `--shell-field-glyph`: the search glyph beside it (`launcher.field_glyph_px`, 20).
    #[token(
        name = "shell-field-glyph",
        input = "--shell-field-glyph-px",
        value = "20px"
    )]
    FieldGlyph,
    /// `--fs-shell-row`: a launcher row's title (`launcher.row_title_px`, 14).
    #[token(name = "fs-shell-row", input = "--shell-row-font", value = "14px")]
    RowTitle,
    /// `--fs-shell-detail`: a launcher row's detail (`launcher.row_detail_px`, 12).
    #[token(
        name = "fs-shell-detail",
        input = "--shell-detail-font",
        value = "12px"
    )]
    RowDetail,
    /// `--fs-shell-tip`: a tooltip's label (`menus.tooltip_font_px`, 12).
    #[token(name = "fs-shell-tip", input = "--shell-tip-font", value = "12px")]
    TipFont,
}

/// A CSS font weight: 400, 500, 700.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

/// The bar's text items and their pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarType {
    /// Text size, 13.
    pub font: Px,
    /// Text weight, 500.
    pub weight: FontWeight,
    /// The hover and open pill's height, 22.
    pub item_height: Px,
    /// The pill's side padding around a text item, 8.
    pub item_padding: Px,
    /// The pill's radius, 5.
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
    /// The selected row's highlight radius, 5.
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
    /// The keys' defaults (settled 2026-09-24, design/13 sections 13.3.1, 13.3.3, 13.3.9; the bar
    /// item and highlight on the size ladder, design/29-SIZING.md section 13).
    fn default() -> Self {
        ShellMetrics {
            bar: BarType {
                font: Px(13.0),
                weight: FontWeight(500),
                item_height: Px(22.0),
                item_padding: Px(8.0),
                item_radius: Px(5.0),
            },
            menu: MenuType {
                font: Px(13.0),
                row: Px(22.0),
                separator_margin: Px(5.0),
                highlight_radius: Px(5.0),
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
            ShellType::BarFont.write(&length(bar.font)),
            ShellType::BarWeight.write(&weight(bar.weight)),
            ShellType::BarItem.write(&length(bar.item_height)),
            ShellType::BarPad.write(&length(bar.item_padding)),
            ShellType::BarRadius.write(&length(bar.item_radius)),
            ShellType::MenuFont.write(&length(menu.font)),
            ShellType::MenuRow.write(&length(menu.row)),
            ShellType::MenuSeparator.write(&length(menu.separator_margin)),
            ShellType::HighlightRadius.write(&length(menu.highlight_radius)),
            ShellType::FieldFont.write(&length(launcher.field_font)),
            ShellType::FieldWeight.write(&weight(launcher.field_weight)),
            ShellType::FieldGlyph.write(&length(launcher.field_glyph)),
            ShellType::RowTitle.write(&length(launcher.row_title)),
            ShellType::RowDetail.write(&length(launcher.row_detail)),
            ShellType::TipFont.write(&length(tip)),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::{ShellMetrics, ShellType};
    use crate::style::tokens::token::TokenScope;
    use ds_core::word::Word;

    #[test]
    fn the_defaults_write_what_the_stylesheet_falls_back_to() {
        let written = ShellMetrics::default().style_attr();
        for token in ShellType::ALL.iter().copied() {
            let want = token.write(token.fallback(TokenScope::BASE));
            assert!(written.contains(&want), "{want} not in {written}");
        }
    }

    #[test]
    fn a_changed_key_moves_only_its_input() {
        let mut metrics = ShellMetrics::default();
        metrics.menu.row = ds_core::geometry::units::Px(24.0);
        let written = metrics.style_attr();
        assert!(written.contains("--shell-menu-row-h:24px;"));
        assert!(written.contains("--shell-menu-font:13px;"));
    }
}
