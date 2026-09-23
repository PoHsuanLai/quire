//! Shadows (design/03-COLOR.md section 10 and 17.2): the two elevation tokens, the plan's
//! `--shadow-pop` and `--shadow-sheet`, C's `--shadow-drag`, and the one-off literals
//! design/04-COMPONENTS.md O-3 says the table must absorb.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
        todo!()
    }

    /// The `box-shadow` value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        todo!()
    }
}
