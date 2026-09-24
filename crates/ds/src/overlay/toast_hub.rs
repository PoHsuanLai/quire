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
    on_undo: Signal<Option<EventHandler<UndoToken>>>,
    hold: Signal<Option<Task>>,
    env: Signal<Env>,
    scope: ScopeId,
}

impl ToastHub {
    /// Show `text`, replacing any toast, and restart the hold.
    pub fn push(&self, text: String, undo: Option<UndoToken>) {
        self.show(text, undo, None);
    }

    /// Show `text` with an undo, replacing any toast, and restart the hold; when the person
    /// undoes this toast, `on_undo` is called with `undo`, so the consumer restores in its own
    /// handler instead of watching [`ToastHub::last_undo`] in an effect.
    ///
    /// A later push replaces the handler with its own (or with none): only the toast on screen
    /// can be undone. The handler belongs to the scope that created it, so push from a
    /// component that outlives the toast (the page, not a row that may be gone by then).
    pub fn push_undoable(&self, text: String, undo: UndoToken, on_undo: EventHandler<UndoToken>) {
        self.show(text, Some(undo), Some(on_undo));
    }

    fn show(
        &self,
        text: String,
        undo: Option<UndoToken>,
        on_undo: Option<EventHandler<UndoToken>>,
    ) {
        let mut handler = self.on_undo;
        handler.set(on_undo);
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

    /// The pull tab was released armed, or clicked: hide at once, call the push's `on_undo`
    /// with the token, and report it.
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
        let mut handler = self.on_undo;
        let handler = handler.take();
        if let (Some(token), Some(handler)) = (undo, handler) {
            handler.call(token);
        }
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
        // Copy the task out first: an `if let` on the peek itself keeps the read guard alive
        // through the body, and the `set` below then panics as already borrowed.
        let running = *self.hold.peek();
        if let Some(running) = running {
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
        on_undo: Signal::new(None),
        hold: Signal::new(None),
        env,
        scope,
    })
}

/// The enclosing `Ds`'s toast manager.
pub fn use_toast_hub() -> ToastHub {
    use_context::<ToastHub>()
}

#[cfg(test)]
mod tests {
    use super::{ToastHub, UndoToken, use_toast_hub_provider};
    use crate::appearance::{Accent, MotionLevel, Resolved, Scheme};
    use crate::material::{BlurState, Material};
    use crate::root::env::{Env, InputModality};
    use dioxus::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// What the handlers saw, and what `undo` returned, in order.
    #[derive(Clone, Default)]
    struct Log(Rc<RefCell<Vec<String>>>);

    impl PartialEq for Log {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    impl Log {
        fn note(&self, line: String) {
            self.0.borrow_mut().push(line);
        }
    }

    #[component]
    fn Script(log: Log) -> Element {
        let env = use_signal(|| Env {
            resolved: Resolved {
                scheme: Scheme::Light,
                accent: Accent::Postmark,
                motion: MotionLevel::Standard,
            },
            scheme: Scheme::Light,
            material: Material::Window,
            blur: BlurState::Unavailable,
            modality: InputModality::Pointer,
        });
        let hub: ToastHub = use_toast_hub_provider(env);
        use_hook(move || {
            let heard = |log: Log| {
                EventHandler::new(move |token: UndoToken| log.note(format!("heard {}", token.0)))
            };
            // An undone push calls its own handler once, with its own token.
            hub.push_undoable("Archived".into(), UndoToken(1), heard(log.clone()));
            log.note(format!("undo {:?}", hub.undo().map(|t| t.0)));
            // Nothing is up now: no second call.
            log.note(format!("undo {:?}", hub.undo().map(|t| t.0)));
            // A later push replaces the handler: the first toast's handler never hears the
            // second's undo.
            hub.push_undoable("Snoozed".into(), UndoToken(2), heard(log.clone()));
            hub.push("Moved".into(), Some(UndoToken(3)));
            log.note(format!("undo {:?}", hub.undo().map(|t| t.0)));
            log.note(format!("last {:?}", hub.last_undo().map(|t| t.0)));
        });
        rsx! {}
    }

    #[test]
    fn an_undo_calls_the_handler_its_push_gave() {
        let log = Log::default();
        let mut dom = VirtualDom::new_with_props(Script, ScriptProps { log: log.clone() });
        dom.rebuild_in_place();
        assert_eq!(
            *log.0.borrow(),
            [
                "heard 1",
                "undo Some(1)",
                "undo None",
                "undo Some(3)",
                "last Some(3)"
            ]
        );
    }
}
