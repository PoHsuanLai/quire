//! The window's keymap as a handle: resolve a key, register actions, draw shortcuts.

use super::source::KeySource;
use super::state::KeyState;
use chordkit::{
    Action, AppAction, AppId, Chord, Conflict, Context, DefaultChord, Keymap, Overrides, Platform,
    Registration,
};
use dioxus::prelude::*;
use ds_core::command::{KeyCap, shortcut_caps, shortcut_text};
use ds_core::vocab::Shortcut;
use std::fmt;
use std::sync::Arc;

/// The keymap of the window, from its [`KeySource`]. `Copy`: hand it to any handler.
///
/// The root (`Ds`) makes one and provides it; read it with `use_keys`. It reloads when the source
/// says its settings changed, and makes every app's registration again over the new keymap.
#[derive(Clone, Copy)]
pub struct Keys {
    state: CopyValue<KeyState>,
    /// Counts reloads, so a view that draws a shortcut redraws when the keymap changes.
    generation: Signal<u64>,
}

impl Keys {
    /// The keymap `source` gives. Call in a component, so the handle lives as long as it.
    pub fn new(source: KeySource) -> Keys {
        Keys {
            state: CopyValue::new(KeyState::load(source)),
            generation: Signal::new(0),
        }
    }

    /// The platform the keymap is for.
    pub fn platform(&self) -> Platform {
        self.state.read().keymap.platform()
    }

    /// The action `key` with `modifiers` held triggers in `context`, if any.
    pub fn resolve(&self, key: &Key, modifiers: Modifiers, context: Context) -> Option<Action> {
        let state = self.state.read();
        ds_core::command::resolve(&state.keymap, key, modifiers, context)
    }

    /// The action a key event triggers in `context`, if any: `Context::TextEntry` in a text
    /// field, `Context::Terminal` in a terminal, `Context::Normal` elsewhere.
    pub fn action_of(&self, event: &KeyboardEvent, context: Context) -> Option<Action> {
        self.resolve(&event.key(), event.modifiers(), context)
    }

    /// Declares `app`'s actions with their portable default chords. A thin wrapper over
    /// [`register`](Keys::register) for an app that forgoes nothing and has no fallbacks.
    pub fn register_actions(
        &self,
        app: &AppId,
        actions: &[(AppAction, DefaultChord)],
    ) -> Result<(), Conflict> {
        let registration = actions
            .iter()
            .fold(Registration::new(app), |registration, (action, chord)| {
                registration.action(action.clone(), *chord)
            });
        self.register(app, &registration)
    }

    /// Declares what `registration` (built for `app`; chordkit offers no way to read the app
    /// back, so it is named again) holds: the app's actions with their defaults and fallbacks,
    /// and the standard actions it does not offer (`Registration::forgo`, `IfOffered` ones only).
    /// All or nothing; the `Conflict` says in plain words why it was refused.
    ///
    /// A registration replaces the app's earlier one, so an app may register changed defaults
    /// (and the same rows again is a no-op). It is made again after every reload.
    pub fn register(&self, app: &AppId, registration: &Registration) -> Result<(), Conflict> {
        let registered = self.state.write_unchecked().register(app, registration);
        if registered.is_ok() {
            self.touch();
        }
        registered
    }

    /// Replaces the person's overrides in a running app, as `AppConfig::with_keymap_overrides`
    /// gave them at launch: they lie again over the conventions, the system's layer and the
    /// app's, every [`register`](Keys::register) is made again (so app actions and forgone
    /// standard actions hold), and every shortcut hint redraws. Empty overrides give the
    /// defaults back, so a reset undoes an earlier change. What the new overrides cannot apply
    /// is kept in [`problems`](Keys::problems).
    pub fn set_overrides(&self, overrides: Overrides) {
        self.state.write_unchecked().set_overrides(overrides);
        self.touch();
    }

    /// The chords that trigger `action` in ordinary window content, best first.
    pub fn chords_of(&self, action: &Action) -> Vec<Chord> {
        let _ = self.generation.read();
        self.state.read().keymap.chords_of(action)
    }

    /// `shortcut` as this keymap's platform shows it (see `ds_core::command::shortcut_text`).
    pub fn text_of(&self, shortcut: &Shortcut) -> String {
        let _ = self.generation.read();
        shortcut_text(&self.state.read().keymap, shortcut)
    }

    /// The caps `shortcut` is drawn on.
    pub fn caps_of(&self, shortcut: &Shortcut) -> Vec<KeyCap> {
        let _ = self.generation.read();
        shortcut_caps(&self.state.read().keymap, shortcut)
    }

    /// Runs `read` over the keymap, for the pure functions that take one.
    pub fn with_keymap<T>(&self, read: impl FnOnce(&Keymap) -> T) -> T {
        read(&self.state.read().keymap)
    }

    /// What went wrong reading the source or registering, in plain words, for a settings screen.
    pub fn problems(&self) -> Vec<String> {
        self.state.read().problems().to_vec()
    }

    /// Keeps `problem` for `problems`.
    pub(super) fn note_problem(&self, problem: String) {
        self.state.write_unchecked().note(problem);
    }

    /// Starts listening to the source; `wake` is called from any thread when it changes.
    pub(super) fn listen(&self, wake: Arc<dyn Fn() + Send + Sync>) {
        let (source, changed) = {
            let state = self.state.read();
            (state.source.clone(), state.changed_flag())
        };
        let watched = source.watch(Box::new(move || {
            changed.store(true, std::sync::atomic::Ordering::Release);
            wake();
        }));
        let mut state = self.state.write_unchecked();
        match watched {
            Ok(watch) => state.hold_watch(watch),
            Err(error) => state.note(error.to_string()),
        }
    }

    /// Reloads the keymap when the source has said it changed. The root calls it every render.
    pub(super) fn reload_if_changed(&self) {
        let changed = self.state.read().take_changed();
        if changed {
            self.state.write_unchecked().reload();
            self.touch();
        }
    }

    fn touch(&self) {
        let mut generation = self.generation;
        generation += 1;
    }
}

impl PartialEq for Keys {
    fn eq(&self, other: &Keys) -> bool {
        self.generation == other.generation
    }
}

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys").finish_non_exhaustive()
    }
}
