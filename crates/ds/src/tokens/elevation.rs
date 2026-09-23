//! Shadows (design/03-COLOR.md section 10 and 17.2): the two elevation tokens, the plan's
//! `--shadow-pop` and `--shadow-sheet`, C's `--shadow-drag`, and the one-off literals
//! design/04-COMPONENTS.md O-3 says the table must absorb.
//!
//! Values are design/03-COLOR.md section 10 (`S`) and C's `--shadow-drag` (section 16);
//! `--shadow-pop` and `--shadow-sheet` take section 17.2's proposed values, and the names past
//! the plan's four are proposed (FINDINGS F13).

use super::name::VarName;
use crate::appearance::Scheme;

/// One shadow token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shadow {
    /// `--shadow-1`: resting raised things.
    One,
    /// `--shadow-2`: hovered rows, the strip, toasts.
    Two,
    /// `--shadow-pop`: menus, hover cards, popovers.
    Pop,
    /// `--shadow-sheet`: peek, sheets, the command menu.
    Sheet,
    /// `--shadow-drag`: the drag ghost.
    Drag,
    /// `--shadow-bubble`: the selection bubble.
    Bubble,
    /// `--shadow-current`: the current sidebar item and the pressed tile.
    Current,
    /// `--shadow-pill-inset`: the command pill's highlight.
    PillInset,
    /// `--shadow-window`: the window.
    Window,
    /// `--shadow-card`: the card.
    Card,
    /// `--shadow-handle`: the editor handle and the slider thumb.
    Handle,
    /// `--shadow-mark`: a provider mark's ring.
    Mark,
    /// `--shadow-mark-tile`: a provider mark on an account tile.
    MarkTile,
}

impl Shadow {
    /// Every shadow, in stylesheet order.
    pub const ALL: [Shadow; 13] = [
        Shadow::One,
        Shadow::Two,
        Shadow::Pop,
        Shadow::Sheet,
        Shadow::Drag,
        Shadow::Bubble,
        Shadow::Current,
        Shadow::PillInset,
        Shadow::Window,
        Shadow::Card,
        Shadow::Handle,
        Shadow::Mark,
        Shadow::MarkTile,
    ];

    /// The custom property: `--shadow-1`, `--shadow-pop`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            Shadow::One => "--shadow-1",
            Shadow::Two => "--shadow-2",
            Shadow::Pop => "--shadow-pop",
            Shadow::Sheet => "--shadow-sheet",
            Shadow::Drag => "--shadow-drag",
            Shadow::Bubble => "--shadow-bubble",
            Shadow::Current => "--shadow-current",
            Shadow::PillInset => "--shadow-pill-inset",
            Shadow::Window => "--shadow-window",
            Shadow::Card => "--shadow-card",
            Shadow::Handle => "--shadow-handle",
            Shadow::Mark => "--shadow-mark",
            Shadow::MarkTile => "--shadow-mark-tile",
        })
    }

    /// The `box-shadow` value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        let dark = scheme == Scheme::Dark;
        match self {
            Shadow::One if dark => "0 1px 0 rgba(255,255,255,.05) inset,0 2px 4px rgba(0,0,0,.45)",
            Shadow::One => "0 1px 0 rgba(255,255,255,.7) inset,0 1px 2px rgba(26,30,26,.1)",
            Shadow::Two if dark => {
                "0 1px 0 rgba(255,255,255,.05) inset,0 10px 22px -8px rgba(0,0,0,.7)"
            }
            Shadow::Two => "0 1px 0 rgba(255,255,255,.7) inset,0 6px 16px -6px rgba(26,30,26,.3)",
            // Hover card and floating menu (`S:400`, `S:666`); section 17.2 names it.
            Shadow::Pop => "0 18px 40px -16px rgba(0,0,0,.45)",
            // Peek (`S:224`) in light; the command menu's (`S:233`) deeper drop in dark.
            Shadow::Sheet if dark => "0 30px 60px -20px rgba(0,0,0,.7)",
            Shadow::Sheet => "0 24px 50px -18px rgba(0,0,0,.55)",
            Shadow::Drag if dark => "0 22px 34px -14px rgba(0,0,0,.8)",
            Shadow::Drag => "0 18px 30px -12px rgba(26,30,26,.4)",
            Shadow::Bubble => "0 12px 28px -12px rgba(0,0,0,.45)",
            Shadow::Current => "0 1px 0 rgba(255,255,255,.4) inset,0 2px 6px -3px rgba(0,0,0,.25)",
            Shadow::PillInset => "0 1px 0 rgba(255,255,255,.35) inset",
            Shadow::Window => "0 18px 40px -22px rgba(0,0,0,.45)",
            Shadow::Card => "0 0 0 1px rgba(0,0,0,.06),0 10px 30px -14px rgba(0,0,0,.45)",
            Shadow::Handle => "0 0 0 1px rgba(0,0,0,.25),0 3px 8px rgba(0,0,0,.35)",
            Shadow::Mark => "0 0 0 1px rgba(0,0,0,.08)",
            Shadow::MarkTile => "0 0 0 1.5px var(--f-pill),0 1px 2px rgba(0,0,0,.2)",
        }
    }
}
