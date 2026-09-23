//! The undo toast's state: one visible at a time, each push restarting the 5200 ms hold
//! (design/04-COMPONENTS.md section 23, design/06-INTERACTIONS.md section 9).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use dioxus::prelude::*;

/// What an undo would restore, as the consumer's own token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UndoToken(pub u64);

/// The toast as it is now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ToastState {
    /// Below the card edge.
    #[default]
    Hidden,
    /// Up, saying `text`, with an undo when there is one.
    Shown {
        /// What just happened: "Archived".
        text: String,
        /// What pulling the tab undoes.
        undo: Option<UndoToken>,
    },
}

/// The toast manager, provided as context by `Ds` and rendered by `ToastHost`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToastHub {
    state: Signal<ToastState>,
    undone: Signal<Option<UndoToken>>,
}

impl ToastHub {
    /// Show `text`, replacing any toast, and restart the hold.
    pub fn push(&self, text: String, undo: Option<UndoToken>) {
        todo!()
    }

    /// The toast as it is now.
    pub fn state(&self) -> ToastState {
        todo!()
    }

    /// The pull tab was released armed, or clicked: hide at once and report the token.
    pub fn undo(&self) -> Option<UndoToken> {
        todo!()
    }

    /// Hide at once.
    pub fn hide(&self) {
        todo!()
    }
}

/// The enclosing `Ds`'s toast manager.
pub fn use_toast_hub() -> ToastHub {
    todo!()
}
