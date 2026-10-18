//! The widths of the sheets that are not the settings sheet's 560: an alert's column and the
//! password prompt's, as tokens so the sheet's stylesheet reads them and a look can move them.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One of the narrow sheets' widths, as a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "sheet-", kind = fixed)]
pub enum SheetSize {
    /// `--sheet-narrow-w`: a password prompt's column, 340.
    #[token(name = "narrow-w", value = "340px")]
    Narrow,
    /// `--sheet-alert-w`: an alert's column, 260: the width of a Sonoma `NSAlert`.
    #[token(name = "alert-w", value = "260px")]
    Alert,
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
