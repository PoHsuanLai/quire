//! The list row's fixed sizes (design/30-CATALOGUE.md section 1.6: "table and list rows 24
//! (compact) / 44 (settings)"; design/29-SIZING.md section 4: the settings row is 44 with a 34
//! leading avatar). design/34-MODERN-LOOK.md section 2.3 (step 2) moves the settings row to 48, the compact row to 32,
//! the avatar to 32 and adds the 28 icon tile and the 12 gap to the label. `Row` and `SkeletonRow` both read these, so a placeholder row is exactly as
//! tall as the row it stands in for (design/30 R3: one implementation per concept).

use crate::tokens::control_size::SidebarSize;
use crate::tokens::size_scale::WholePx;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// The settings-density row's measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowScale {
    /// The settings row's minimum height, 48.
    pub settings_height: WholePx,
    /// The settings row's leading avatar, thumbnail or mark: a 32 circle.
    pub avatar: WholePx,
    /// The compact (table and source-list) row's height, 32.
    pub compact_height: WholePx,
    /// The rounded icon tile leading a grouped-list row, a 28 square.
    pub icon_tile: WholePx,
    /// The gap between a row's leading tile or avatar and its label, 12.
    pub tile_gap: WholePx,
}

/// The settled numbers.
pub const ROW_SCALE: RowScale = RowScale {
    settings_height: WholePx(48),
    avatar: WholePx(32),
    compact_height: WholePx(32),
    icon_tile: WholePx(28),
    tile_gap: WholePx(12),
};

/// One of the row's sizes, as a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = row_size_css)]
pub enum RowSize {
    /// `--row-settings-h`: the settings row's minimum height.
    #[token(name = "row-settings-h")]
    SettingsHeight,
    /// `--row-avatar`: the settings row's leading element, a square this wide.
    #[token(name = "row-avatar")]
    Avatar,
    /// `--row-compact-h`: the compact row's height.
    #[token(name = "row-compact-h")]
    CompactHeight,
    /// `--row-icon-tile`: the icon tile's side.
    #[token(name = "row-icon-tile")]
    IconTile,
    /// `--row-tile-gap`: the gap from the leading element to the label.
    #[token(name = "row-tile-gap")]
    TileGap,
}

/// The size a row token holds, as the stylesheet writes it.
fn row_size_css(token: RowSize, _scope: TokenScope) -> CssValue {
    let size = match token {
        RowSize::SettingsHeight => ROW_SCALE.settings_height,
        RowSize::Avatar => ROW_SCALE.avatar,
        RowSize::CompactHeight => ROW_SCALE.compact_height,
        RowSize::IconTile => ROW_SCALE.icon_tile,
        RowSize::TileGap => ROW_SCALE.tile_gap,
    };
    CssValue::computed(size.css())
}

/// A source list's row height at each [`SidebarSize`], as a token, so the list's stylesheet reads
/// the ladder (design/34 section 2.2: 28, 32, 36).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = sidebar_row_css)]
pub enum SidebarRowSize {
    /// `--sidebar-row-h-s`: the Small sidebar's row.
    #[token(name = "sidebar-row-h-s")]
    Small,
    /// `--sidebar-row-h-m`: the Medium sidebar's row.
    #[token(name = "sidebar-row-h-m")]
    Medium,
    /// `--sidebar-row-h-l`: the Large sidebar's row.
    #[token(name = "sidebar-row-h-l")]
    Large,
}

/// The height a sidebar row token holds, as the stylesheet writes it.
fn sidebar_row_css(token: SidebarRowSize, _scope: TokenScope) -> CssValue {
    let size = match token {
        SidebarRowSize::Small => SidebarSize::Small,
        SidebarRowSize::Medium => SidebarSize::Medium,
        SidebarRowSize::Large => SidebarSize::Large,
    };
    CssValue::computed(size.row_height().css())
}

/// An icon tile's side at each [`SidebarSize`], as a token: a source-list row's tile sits 4 px
/// in from the row's top and bottom (20, 24, 28 in rows of 28, 32, 36).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = sidebar_tile_css)]
pub enum SidebarTileSize {
    /// `--sidebar-tile-s`: the Small sidebar's tile.
    #[token(name = "sidebar-tile-s")]
    Small,
    /// `--sidebar-tile-m`: the Medium sidebar's tile.
    #[token(name = "sidebar-tile-m")]
    Medium,
    /// `--sidebar-tile-l`: the Large sidebar's tile.
    #[token(name = "sidebar-tile-l")]
    Large,
}

/// The side a sidebar tile token holds, as the stylesheet writes it.
fn sidebar_tile_css(token: SidebarTileSize, _scope: TokenScope) -> CssValue {
    let size = match token {
        SidebarTileSize::Small => SidebarSize::Small,
        SidebarTileSize::Medium => SidebarSize::Medium,
        SidebarTileSize::Large => SidebarSize::Large,
    };
    CssValue::computed(size.tile_size().css())
}

#[cfg(test)]
mod tests {
    use super::{ROW_SCALE, RowSize};
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;

    #[test]
    fn each_token_is_its_scale_value() {
        let css = |token: RowSize| token.css_value(TokenScope::BASE).as_str().to_owned();
        assert_eq!(css(RowSize::SettingsHeight), "48px");
        assert_eq!(css(RowSize::Avatar), "32px");
        assert_eq!(css(RowSize::CompactHeight), "32px");
        assert_eq!(css(RowSize::IconTile), "28px");
        assert_eq!(css(RowSize::TileGap), "12px");
        assert_eq!(RowSize::SettingsHeight.var().as_str(), "--row-settings-h");
        assert_eq!(RowSize::Avatar.var().as_str(), "--row-avatar");
        assert_eq!(RowSize::ALL.len(), 5);
    }

    #[test]
    fn a_sidebar_row_token_is_its_sizes_height() {
        use super::SidebarRowSize;
        use crate::tokens::control_size::SidebarSize;
        let css = |token: SidebarRowSize| token.css_value(TokenScope::BASE).as_str().to_owned();
        assert_eq!(
            css(SidebarRowSize::Small),
            SidebarSize::Small.row_height().css()
        );
        assert_eq!(
            css(SidebarRowSize::Medium),
            SidebarSize::Medium.row_height().css()
        );
        assert_eq!(
            css(SidebarRowSize::Large),
            SidebarSize::Large.row_height().css()
        );
    }

    #[test]
    fn a_sidebar_tile_token_is_its_sizes_tile() {
        use super::SidebarTileSize;
        let css = |token: SidebarTileSize| token.css_value(TokenScope::BASE).as_str().to_owned();
        assert_eq!(css(SidebarTileSize::Small), "20px");
        assert_eq!(css(SidebarTileSize::Medium), "24px");
        assert_eq!(css(SidebarTileSize::Large), "28px");
    }

    #[test]
    fn the_avatar_sits_inside_the_row_with_an_even_margin() {
        let margin = ROW_SCALE.settings_height.0.checked_sub(ROW_SCALE.avatar.0);
        assert!(
            margin.is_some_and(|margin| margin > 0 && margin % 2 == 0),
            "the avatar sits inside the row with the same space above and below"
        );
    }
}
