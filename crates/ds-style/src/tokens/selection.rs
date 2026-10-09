//! How a selected row is painted (design/30-CATALOGUE.md section 3.2, "Selection"): `--sel-bg`
//! under an active window's selection, `--sel-bg-quiet` under the same selection when the window
//! is inactive, and `--sel-ink` for the text on either.
//!
//! The selection fills the row with the accent and writes accent ink over it, and greys it when
//! the window loses focus. Each value names the accent's own properties, so it follows the
//! accent a root selects.

use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One selection paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "sel-", kind = fixed, css = selection_css)]
#[non_exhaustive]
pub enum SelectionToken {
    /// `--sel-bg`: the selected row's fill while its window is active.
    Bg,
    /// `--sel-bg-quiet`: the same fill while the window is inactive.
    #[token(name = "bg-quiet")]
    BgQuiet,
    /// `--sel-ink`: text and glyphs on either fill.
    Ink,
}

/// A selection token as the stylesheet writes it.
fn selection_css(token: SelectionToken, _: TokenScope) -> CssValue {
    CssValue::fixed(match token {
        SelectionToken::Bg => "var(--accent)",
        SelectionToken::BgQuiet => "rgba(128,128,128,.25)",
        SelectionToken::Ink => "var(--accent-ink)",
    })
}
