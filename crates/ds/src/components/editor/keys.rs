//! A key on an [`EditSurface`](crate::components::editor::surface::EditSurface): nothing while the IME composes (it owns the
//! keys), else end a cleared composition, then hand over what the key means.

use crate::components::editor::ctx::SurfaceCtx;
use crate::edit::composition::{Composing, settle};
use crate::edit::input::EditInput;
use crate::edit::keys::{KeyAction, classify};
use dioxus::prelude::*;

pub(crate) fn key_down(ctx: &SurfaceCtx, event: &KeyboardEvent) {
    if ctx.state.composing.get() == Composing::Active {
        return;
    }
    let (idle, ended) = settle(ctx.state.composing.get());
    ctx.state.composing.set(idle);
    ctx.tell(ended);
    let input = match classify(&event.key(), event.modifiers()) {
        KeyAction::Text(text) => EditInput::Text(text),
        KeyAction::Key(key) => EditInput::Key(key),
        KeyAction::Cut => EditInput::Cut,
        KeyAction::Copy => EditInput::Copy,
        KeyAction::Paste => {
            event.prevent_default();
            match ctx.host.edit().and_then(|edit| edit.paste()) {
                Some(pasted) => EditInput::Paste(pasted),
                None => return,
            }
        }
    };
    ctx.on_input.call(input);
}
