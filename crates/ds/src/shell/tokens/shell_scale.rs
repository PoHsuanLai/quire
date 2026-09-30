//! The bar's and the text menus' geometry on the size ladder (design/29-SIZING.md sections 5.1,
//! 5.4 and 13): a 24 bar whose items are Regular controls (22), a status slot 30 wide with a 16
//! glyph, and a menu rounded 8 with its rows inset 5 (the reference's highlight is 5, not
//! concentric with the menu). The item heights, paddings and highlight a settings key can move
//! stay [`ShellMetrics`](super::ShellMetrics) tokens; these are the fixed ones.

use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::size_scale::WholePx;
use ds_style::tokens::token::{CssValue, Token, TokenScope};

/// The bar and text menus' fixed sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShellScale {
    /// The bar's height, 24.
    pub bar: WholePx,
    /// A status item's slot width, 30; its height is a Regular control's.
    pub status_width: WholePx,
    /// A text menu's radius, 8.
    pub menu_radius: WholePx,
    /// A text menu's inset round its rows, 5.
    pub menu_inset: WholePx,
}

/// The settled numbers (design/29 section 13 decision 2; the reference's menu).
pub const SHELL_SCALE: ShellScale = ShellScale {
    bar: WholePx(24),
    status_width: WholePx(30),
    menu_radius: WholePx(8),
    menu_inset: WholePx(5),
};

impl ShellScale {
    /// A bar item's height: a Regular control, 22.
    pub fn item(self) -> WholePx {
        ControlSize::Regular.scale().height
    }
}

/// One of the shell's sizes on the ladder, as a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = shell_size_css)]
pub enum ShellSize {
    /// `--shell-bar-h`: the bar's height.
    #[token(name = "shell-bar-h")]
    Bar,
    /// `--bar-status-w`: a status item's width.
    #[token(name = "bar-status-w")]
    StatusWidth,
    /// `--r-shell-menu`: a text menu's radius.
    #[token(name = "r-shell-menu")]
    MenuRadius,
    /// `--shell-menu-inset`: how far a menu row is inset from the menu's edge.
    #[token(name = "shell-menu-inset")]
    MenuInset,
}

/// The size a shell token holds, as the stylesheet writes it.
fn shell_size_css(token: ShellSize, _scope: TokenScope) -> CssValue {
    let size = match token {
        ShellSize::Bar => SHELL_SCALE.bar,
        ShellSize::StatusWidth => SHELL_SCALE.status_width,
        ShellSize::MenuRadius => SHELL_SCALE.menu_radius,
        ShellSize::MenuInset => SHELL_SCALE.menu_inset,
    };
    CssValue::computed(size.css())
}
