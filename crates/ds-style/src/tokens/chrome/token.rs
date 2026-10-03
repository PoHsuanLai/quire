//! The window chrome's geometry as custom properties, for `window_frame.css` to read.

use super::scale::CHROME_SCALE;
use crate::tokens::size_scale::WholePx;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One length of the window chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "chrome-", kind = fixed, css = chrome_css)]
pub enum ChromeToken {
    /// `--chrome-titlebar-height` 28.
    TitlebarHeight,
    /// `--chrome-light-size` 12: a traffic light's diameter.
    LightSize,
    /// `--chrome-light-gap` 8: between two lights.
    LightGap,
    /// `--chrome-light-inset` 13: the first light's distance from the window's left edge.
    LightInset,
    /// `--chrome-title-inset` 84: the title's distance from each side.
    TitleInset,
    /// `--chrome-edge` 4: a side's resize zone.
    Edge,
    /// `--chrome-edge-corner` 12: a corner's resize zone.
    EdgeCorner,
}

impl ChromeToken {
    /// The length in logical pixels.
    pub fn length(self) -> WholePx {
        match self {
            ChromeToken::TitlebarHeight => CHROME_SCALE.titlebar_height,
            ChromeToken::LightSize => CHROME_SCALE.light_size,
            ChromeToken::LightGap => CHROME_SCALE.light_gap,
            ChromeToken::LightInset => CHROME_SCALE.light_inset,
            ChromeToken::TitleInset => CHROME_SCALE.title_inset,
            ChromeToken::Edge => CHROME_SCALE.edge,
            ChromeToken::EdgeCorner => CHROME_SCALE.edge_corner,
        }
    }
}

/// A chrome length as the stylesheet writes it.
fn chrome_css(token: ChromeToken, _scope: TokenScope) -> CssValue {
    CssValue::computed(token.length().css())
}

#[cfg(test)]
mod tests {
    use super::ChromeToken;
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;

    /// Each token's property and the value the window frame's literals had.
    const CASES: &[(ChromeToken, &str, &str)] = &[
        (
            ChromeToken::TitlebarHeight,
            "--chrome-titlebar-height",
            "28px",
        ),
        (ChromeToken::LightSize, "--chrome-light-size", "12px"),
        (ChromeToken::LightGap, "--chrome-light-gap", "8px"),
        (ChromeToken::LightInset, "--chrome-light-inset", "13px"),
        (ChromeToken::TitleInset, "--chrome-title-inset", "84px"),
        (ChromeToken::Edge, "--chrome-edge", "4px"),
        (ChromeToken::EdgeCorner, "--chrome-edge-corner", "12px"),
    ];

    #[test]
    fn every_token_writes_its_property_and_length() {
        for (token, var, value) in CASES {
            assert_eq!(token.var().as_str(), *var, "{token:?} property");
            assert_eq!(
                token.css_value(TokenScope::BASE).as_str(),
                *value,
                "{token:?} value"
            );
        }
        assert_eq!(CASES.len(), ChromeToken::ALL.len(), "every token has a row");
    }
}
