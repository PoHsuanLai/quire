//! The list row's fixed sizes (design/30-CATALOGUE.md section 1.6: "table and list rows 24
//! (compact) / 44 (settings)"; design/29-SIZING.md section 4: the settings row is 44 with a 34
//! leading avatar). `Row` and `SkeletonRow` both read these, so a placeholder row is exactly as
//! tall as the row it stands in for (design/30 R3: one implementation per concept).

use crate::tokens::size_scale::WholePx;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// The settings-density row's measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowScale {
    /// The settings row's minimum height, 44.
    pub settings_height: WholePx,
    /// The settings row's leading avatar, thumbnail or mark: a 34 square.
    pub avatar: WholePx,
}

/// The settled numbers.
pub const ROW_SCALE: RowScale = RowScale {
    settings_height: WholePx(44),
    avatar: WholePx(34),
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
}

/// The size a row token holds, as the stylesheet writes it.
fn row_size_css(token: RowSize, _scope: TokenScope) -> CssValue {
    let size = match token {
        RowSize::SettingsHeight => ROW_SCALE.settings_height,
        RowSize::Avatar => ROW_SCALE.avatar,
    };
    CssValue::computed(size.css())
}

#[cfg(test)]
mod tests {
    use super::{ROW_SCALE, RowSize};
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;

    #[test]
    fn each_token_is_its_scale_value() {
        let css = |token: RowSize| token.css_value(TokenScope::BASE).as_str().to_owned();
        assert_eq!(css(RowSize::SettingsHeight), "44px");
        assert_eq!(css(RowSize::Avatar), "34px");
        assert_eq!(RowSize::SettingsHeight.var().as_str(), "--row-settings-h");
        assert_eq!(RowSize::Avatar.var().as_str(), "--row-avatar");
        assert_eq!(RowSize::ALL.len(), 2);
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
