//! The widths of the sheets that are not the settings sheet's 560: an alert's column and the
//! password prompt's, as tokens so the sheet's stylesheet reads them and a look can move them.

use crate::tokens::size_scale::WholePx;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// The narrow sheets' widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SheetScale {
    /// A password prompt's column, 340.
    pub narrow: WholePx,
    /// An alert's column, 260: the width of a Sonoma `NSAlert`.
    pub alert: WholePx,
}

/// The settled numbers.
pub const SHEET_SCALE: SheetScale = SheetScale {
    narrow: WholePx(340),
    alert: WholePx(260),
};

/// One of the narrow sheets' widths, as a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "sheet-", kind = fixed, css = sheet_size_css)]
pub enum SheetSize {
    /// `--sheet-narrow-w`: a password prompt's column.
    #[token(name = "narrow-w")]
    Narrow,
    /// `--sheet-alert-w`: an alert's column.
    #[token(name = "alert-w")]
    Alert,
}

/// The width a sheet token holds, as the stylesheet writes it.
fn sheet_size_css(token: SheetSize, _scope: TokenScope) -> CssValue {
    let width = match token {
        SheetSize::Narrow => SHEET_SCALE.narrow,
        SheetSize::Alert => SHEET_SCALE.alert,
    };
    CssValue::computed(width.css())
}

#[cfg(test)]
mod tests {
    use super::SheetSize;
    use crate::tokens::token::{Token, TokenScope};

    #[test]
    fn an_alert_is_260_and_a_password_prompt_340() {
        let css = |token: SheetSize| token.css_value(TokenScope::BASE).to_string();
        assert_eq!(SheetSize::Alert.var().as_str(), "--sheet-alert-w");
        assert_eq!(css(SheetSize::Alert), "260px");
        assert_eq!(css(SheetSize::Narrow), "340px");
    }
}
