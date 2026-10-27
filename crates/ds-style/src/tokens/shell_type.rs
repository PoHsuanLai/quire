//! The shell type scale (the macOS polish pass, 2026-09-24): the bar's items, text menus, the
//! launcher and tooltips, each size a tuned token whose default is declared in the stylesheet
//! (`var(--shell-menu-font,13px)`); a look or a style sheet overrides the input. The defaults are
//! the macOS numbers design/13-BEHAVIOUR-menus-windows.md section 13.2 records (menu bar text
//! 13 pt, menu rows about 22 pt, 13 pt menu text) and the launcher's Spotlight-like scale
//! (design/13 section 13.3.9); `MenuBarItem`, `Menu`, `CommandPalette`
//! in a surface and `Tooltip` read them, so a consumer gets them with no prop.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One token of the shell type scale, each a tuned token a look or style sheet can move.
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
    /// `--shell-menu-row`: a text menu row's height (`menus.item_height_px`, 28).
    #[token(name = "shell-menu-row", input = "--shell-menu-row-h", value = "28px")]
    MenuRow,
    /// `--shell-menu-sep`: the margin above and below a separator (`menus.separator_margin_px`, 5).
    #[token(name = "shell-menu-sep", input = "--shell-menu-sep-m", value = "5px")]
    MenuSeparator,
    /// `--r-shell-highlight`: the selected row's inset highlight (`menus.highlight_radius_px`, 8: the
    /// reference's, design/29-SIZING.md section 5.4).
    #[token(
        name = "r-shell-highlight",
        input = "--shell-highlight-radius",
        value = "8px"
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

#[cfg(test)]
mod tests {
    use super::ShellType;
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;

    #[test]
    fn every_input_reads_with_its_default_behind_it() {
        for token in ShellType::ALL.iter().copied() {
            let value = token.css_value(TokenScope::BASE).to_string();
            assert!(value.starts_with("var(--shell-"), "{token:?}: {value}");
            assert!(value.contains(','), "{token:?}: {value}");
        }
    }

    #[test]
    fn the_highlight_radius_default_is_the_concentric_menu_row() {
        let concentric = crate::tokens::shell_scale::SHELL_SCALE.menu_radius.0
            - crate::tokens::shell_scale::SHELL_SCALE.menu_inset.0;
        assert_eq!(
            ShellType::HighlightRadius
                .css_value(TokenScope::BASE)
                .to_string(),
            format!("var(--shell-highlight-radius,{concentric}px)")
        );
    }
}
