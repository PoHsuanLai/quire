//! A key handler that speaks actions: the one line an app puts on a surface.

use super::handle::Keys;
use chordkit::{Action, Context};
use dioxus::prelude::KeyboardEvent;

/// Whether the app took the action. Only a taken action consumes the key event; one the app
/// ignores goes on to the focused widget as usual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionTaken {
    /// The app handled it: the key event is consumed.
    Yes,
    /// The app did not: the key event goes on.
    No,
}

/// An `onkeydown` handler that resolves each key in `context` and hands the action to `handle`.
///
/// ```ignore
/// let keys = use_keys();
/// rsx! { div { onkeydown: on_action(keys, Context::Normal, move |action| match action {
///     Action::Standard(StandardAction::Save) => { save(); ActionTaken::Yes }
///     _ => ActionTaken::No,
/// }) } }
/// ```
pub fn on_action(
    keys: Keys,
    context: Context,
    mut handle: impl FnMut(Action) -> ActionTaken + 'static,
) -> impl FnMut(KeyboardEvent) + 'static {
    move |event: KeyboardEvent| {
        let Some(action) = keys.action_of(&event, context) else {
            return;
        };
        if handle(action) == ActionTaken::Yes {
            event.prevent_default();
            event.stop_propagation();
        }
    }
}
