//! The keymap a window holds and everything that must survive a reload: the apps' registrations,
//! the problems worth telling a settings screen, and the source's watch.

use chordkit::{AppAction, AppId, Conflict, DefaultChord, Keymap, Watch};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::source::KeySource;

/// One app's registration, kept to be made again after a reload.
#[derive(Debug, Clone, PartialEq)]
struct Registration {
    app: AppId,
    actions: Vec<(AppAction, DefaultChord)>,
}

/// The window's keymap and what it was built from.
#[derive(Debug)]
pub(super) struct KeyState {
    pub(super) source: KeySource,
    pub(super) keymap: Keymap,
    registered: Vec<Registration>,
    problems: Vec<String>,
    /// Set from the source's thread when its settings changed.
    changed: Arc<AtomicBool>,
    watch: Option<Watch>,
}

impl KeyState {
    /// The keymap `source` gives; its conventions, with the reason told, when it cannot be read.
    pub(super) fn load(source: KeySource) -> KeyState {
        let (keymap, problems) = loaded(&source);
        KeyState {
            source,
            keymap,
            registered: Vec::new(),
            problems,
            changed: Arc::new(AtomicBool::new(false)),
            watch: None,
        }
    }

    /// Registers `actions` of `app`, and remembers them for the next reload.
    pub(super) fn register(
        &mut self,
        app: &AppId,
        actions: &[(AppAction, DefaultChord)],
    ) -> Result<(), Conflict> {
        self.keymap.register(app, actions)?;
        let known = self
            .registered
            .iter()
            .any(|r| r.app == *app && r.actions.as_slice() == actions);
        if !known {
            self.registered.push(Registration {
                app: app.clone(),
                actions: actions.to_vec(),
            });
        }
        Ok(())
    }

    /// Reads the source again and makes every registration again over it.
    pub(super) fn reload(&mut self) {
        let (keymap, problems) = loaded(&self.source);
        self.keymap = keymap;
        self.problems = problems;
        for registration in self.registered.clone() {
            if let Err(conflict) = self
                .keymap
                .register(&registration.app, &registration.actions)
            {
                self.problems.push(conflict.to_string());
            }
        }
    }

    /// Whether the source said its settings changed since the last call.
    pub(super) fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }

    /// The flag the source's change callback sets.
    pub(super) fn changed_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.changed)
    }

    pub(super) fn hold_watch(&mut self, watch: Watch) {
        self.watch = Some(watch);
    }

    pub(super) fn problems(&self) -> &[String] {
        &self.problems
    }

    pub(super) fn note(&mut self, problem: String) {
        self.problems.push(problem);
    }
}

fn loaded(source: &KeySource) -> (Keymap, Vec<String>) {
    match source.load() {
        Ok(keymap) => (keymap, Vec::new()),
        Err(error) => (
            Keymap::conventional(source.platform()),
            vec![error.to_string()],
        ),
    }
}
