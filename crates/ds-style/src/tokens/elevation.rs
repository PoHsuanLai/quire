//! Shadows (design/03-COLOR.md section 10 and 17.2): the two elevation tokens, the plan's
//! `--shadow-pop` and `--shadow-sheet`, C's `--shadow-drag`, and the one-off literals
//! design/04-COMPONENTS.md O-3 says the table must absorb.
//!
//! Values are design/03-COLOR.md section 10 (`S`) and C's `--shadow-drag` (section 16);
//! `--shadow-pop` and `--shadow-sheet` take section 17.2's proposed values, and the names past
//! the plan's four are proposed.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One shadow token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "shadow-", kind = fixed)]
pub enum Shadow {
    /// `--shadow-1`: resting raised things.
    #[token(
        name = "1",
        light = "0 var(--hair) 0 rgba(255,255,255,.7) inset,0 1px 2px rgba(26,30,26,.1)",
        dark = "0 var(--hair) 0 rgba(255,255,255,.05) inset,0 2px 4px rgba(0,0,0,.45)"
    )]
    One,
    /// `--shadow-2`: hovered rows, the strip, toasts.
    #[token(
        name = "2",
        light = "0 var(--hair) 0 rgba(255,255,255,.7) inset,0 6px 16px -6px rgba(26,30,26,.3)",
        dark = "0 var(--hair) 0 rgba(255,255,255,.05) inset,0 10px 22px -8px rgba(0,0,0,.7)"
    )]
    Two,
    /// `--shadow-pop`: menus, hover cards, popovers.
    #[token(value = "0 18px 40px -16px rgba(0,0,0,.45)")]
    Pop,
    /// `--shadow-sheet`: peek, sheets, the command menu.
    #[token(
        light = "0 24px 50px -18px rgba(0,0,0,.55)",
        dark = "0 30px 60px -20px rgba(0,0,0,.7)"
    )]
    Sheet,
    /// `--shadow-drag`: the drag ghost.
    #[token(
        light = "0 18px 30px -12px rgba(26,30,26,.4)",
        dark = "0 22px 34px -14px rgba(0,0,0,.8)"
    )]
    Drag,
    /// `--shadow-current`: the current sidebar item and the pressed tile.
    #[token(value = "0 var(--hair) 0 rgba(255,255,255,.4) inset,0 2px 6px -3px rgba(0,0,0,.25)")]
    Current,
    /// `--shadow-pill-inset`: the command pill's highlight.
    #[token(value = "0 var(--hair) 0 rgba(255,255,255,.35) inset")]
    PillInset,
    /// `--shadow-window`: an app window, a contact and a wide ambient drop.
    #[token(
        light = "0 1px 3px rgba(0,0,0,.12),0 24px 64px -16px rgba(0,0,0,.4)",
        dark = "0 1px 3px rgba(0,0,0,.4),0 24px 64px -16px rgba(0,0,0,.66)"
    )]
    Window,
    /// `--shadow-card`: the card.
    #[token(value = "0 0 0 var(--hair) rgba(0,0,0,.06),0 10px 30px -14px rgba(0,0,0,.45)")]
    Card,
    /// `--shadow-handle`: the editor handle and the slider thumb.
    #[token(value = "0 0 0 var(--hair) rgba(0,0,0,.25),0 3px 8px rgba(0,0,0,.35)")]
    Handle,
    /// `--shadow-mark`: a provider mark's ring.
    #[token(value = "0 0 0 var(--hair) rgba(0,0,0,.08)")]
    Mark,
    /// `--shadow-mark-tile`: a provider mark on an account tile.
    #[token(value = "0 0 0 1.5px var(--f-pill),0 1px 2px rgba(0,0,0,.2)")]
    MarkTile,
}
