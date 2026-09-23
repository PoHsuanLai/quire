//! The undo toast's state: one visible at a time, each push restarting the 5200 ms hold
//! (design/04-COMPONENTS.md section 23, design/06-INTERACTIONS.md section 9).

use crate::root::env::Env;
use crate::time::{sleep, spawn_in};
use crate::tokens::DelayToken;
use dioxus::core::{Task, current_scope_id};
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
    hold: Signal<Option<Task>>,
    env: Signal<Env>,
    scope: ScopeId,
}

impl ToastHub {
    /// Show `text`, replacing any toast, and restart the hold.
    pub fn push(&self, text: String, undo: Option<UndoToken>) {
        let mut state = self.state;
        state.set(ToastState::Shown { text, undo });
        self.stop_hold();
        let hold = DelayToken::ToastHold.delay(self.env.peek().resolved.motion);
        let hub = *self;
        let started = spawn_in(self.scope, async move {
            sleep(hold).await;
            let mut task = hub.hold;
            task.set(None);
            hub.hide();
        });
        let mut task = self.hold;
        task.set(Some(started));
    }

    /// The toast as it is now.
    pub fn state(&self) -> ToastState {
        self.state.read().clone()
    }

    /// The pull tab was released armed, or clicked: hide at once and report the token.
    ///
    /// `None`, and nothing hidden, when no toast is up. The token is also kept as the last
    /// undo, for a consumer that watches rather than handles the release.
    pub fn undo(&self) -> Option<UndoToken> {
        let ToastState::Shown { undo, .. } = self.state.peek().clone() else {
            return None;
        };
        self.hide();
        let mut undone = self.undone;
        undone.set(undo);
        undo
    }

    /// The token the last undo reported.
    pub fn last_undo(&self) -> Option<UndoToken> {
        (self.undone)()
    }

    /// Hide at once.
    pub fn hide(&self) {
        self.stop_hold();
        let mut state = self.state;
        if *state.peek() != ToastState::Hidden {
            state.set(ToastState::Hidden);
        }
    }

    fn stop_hold(&self) {
        if let Some(running) = *self.hold.peek() {
            running.cancel();
            let mut task = self.hold;
            task.set(None);
        }
    }
}

/// A new hub for `Ds` to provide, timing its hold at the root's motion level.
pub(crate) fn use_toast_hub_provider(env: Signal<Env>) -> ToastHub {
    let scope = use_hook(current_scope_id);
    use_context_provider(|| ToastHub {
        state: Signal::new(ToastState::Hidden),
        undone: Signal::new(None),
        hold: Signal::new(None),
        env,
        scope,
    })
}

/// The enclosing `Ds`'s toast manager.
pub fn use_toast_hub() -> ToastHub {
    use_context::<ToastHub>()
}
