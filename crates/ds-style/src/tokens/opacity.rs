//! Resting opacities a keyframe ends on, named so a keyframe and the element that rests there
//! agree.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One resting opacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed)]
#[non_exhaustive]
pub enum OpacityToken {
    /// `--veil` .16: an ink layer laid over content to push it back (C's scrim, `C:1055`: `ink`
    /// at .16), where `fade-in` ends. Not `--scrim`, which is a colour (black at .22) painted at
    /// full opacity and faded by `fade`.
    #[token(value = ".16")]
    Veil,
    /// `--pane-dim` .6: a pane of a split view that does not hold the focus, when the view dims
    /// them (`UnfocusedPanes::Dim`), so the one being typed into stands out.
    #[token(value = ".6")]
    PaneDim,
}

#[cfg(test)]
mod tests {
    use super::OpacityToken;
    use crate::tokens::token::{Token, TokenScope};

    #[test]
    fn the_pane_dim_leaves_an_unfocused_pane_readable() {
        assert_eq!(OpacityToken::PaneDim.var().as_str(), "--pane-dim");
        assert_eq!(
            OpacityToken::PaneDim.css_value(TokenScope::BASE).as_str(),
            ".6"
        );
    }

    #[test]
    fn the_veil_is_c_s_scrim_opacity() {
        assert_eq!(OpacityToken::Veil.var().as_str(), "--veil");
        assert_eq!(
            OpacityToken::Veil.css_value(TokenScope::BASE).as_str(),
            ".16"
        );
    }
}
