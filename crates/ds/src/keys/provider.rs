//! Providing the keymap at the root and reading it anywhere below.

use super::handle::Keys;
use super::source::KeySource;
use chordkit::{AppAction, AppId, Conflict, DefaultChord, Platform};
use dioxus::prelude::*;

/// The root's keymap: the enclosing root's when there is one, else a new one from the
/// [`KeySource`] the launcher provided (our desktop's conventions without one). Reloads it when
/// the source said its settings changed. `Ds` calls this; an app does not.
pub fn use_keys_provider() -> Keys {
    let keys = use_hook(|| {
        if let Some(enclosing) = try_consume_context::<Keys>() {
            return enclosing;
        }
        let source = try_consume_context::<KeySource>().unwrap_or_default();
        let keys = Keys::new(source);
        keys.listen(dioxus::core::schedule_update());
        provide_context(keys)
    });
    keys.reload_if_changed();
    keys
}

/// The keymap of the enclosing `Ds`. Under no root (a component drawn on its own) it is our
/// desktop's conventions, so a component never needs to ask whether there is one.
pub fn use_keys() -> Keys {
    use_hook(|| try_consume_context::<Keys>().unwrap_or_else(|| Keys::new(KeySource::default())))
}

/// The platform of the enclosing `Ds`'s keymap: all a view needs to draw a shortcut or read a
/// wheel zoom.
pub fn use_platform() -> Platform {
    use_hook(|| {
        try_consume_context::<Keys>()
            .map_or_else(|| KeySource::default().platform(), |keys| keys.platform())
    })
}

/// Declares `app`'s actions once, at its first render; the `Conflict` says why a chord was
/// refused. Registered again by the keymap after each reload.
pub fn use_register_actions(
    app: &AppId,
    actions: &[(AppAction, DefaultChord)],
) -> Result<(), Conflict> {
    let keys = use_keys();
    use_hook(|| keys.register_actions(app, actions))
}
