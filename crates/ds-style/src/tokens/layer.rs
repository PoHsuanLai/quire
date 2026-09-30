//! Z-order inside a window (design/01-LAYOUT.md section 12). The plan names the ends,
//! `--z-raise` 1 and `--z-drag` 50; the layers between are named after what sits on them.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One stacking layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "z-", kind = fixed)]
pub enum ZLayer {
    /// `--z-scene` -2: the frame's gradient layers.
    #[token(value = "-2")]
    Scene,
    /// `--z-raise` 1: a lifted row or control.
    #[token(value = "1")]
    Raise,
    /// `--z-link-pill` 7.
    #[token(value = "7")]
    LinkPill,
    /// `--z-toast` 8.
    #[token(value = "8")]
    Toast,
    /// `--z-send-pill` 9: also the floating composer.
    #[token(value = "9")]
    SendPill,
    /// `--z-scrim` 10.
    #[token(value = "10")]
    Scrim,
    /// `--z-peek` 11.
    #[token(value = "11")]
    Peek,
    /// `--z-focus-page` 12: the focus-mode composer page.
    #[token(value = "12")]
    FocusPage,
    /// `--z-edge` 14: the left edge strip.
    #[token(value = "14")]
    Edge,
    /// `--z-side-peek` 15.
    #[token(value = "15")]
    SidePeek,
    /// `--z-palette` 20: the command menu.
    #[token(value = "20")]
    Palette,
    /// `--z-card` 30: hover cards.
    #[token(value = "30")]
    Card,
    /// `--z-menu` 40: floating menus and the zZ floater.
    #[token(value = "40")]
    Menu,
    /// `--z-drag` 50: the drag ghost.
    #[token(value = "50")]
    Drag,
}
