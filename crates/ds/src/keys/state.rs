use chordkit::{AppId, Conflict, Keymap, Registration, Watch};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::source::KeySource;

/// One app's registration, kept to be made again after a reload. An app has at most one.
#[derive(Debug, Clone, PartialEq)]
struct Registered {
    app: AppId,
    registration: Registration,
}

/// The window's keymap and what it was built from.
#[derive(Debug)]
pub(super) struct KeyState {
    pub(super) source: KeySource,
    /// What the source gives with the app's overrides on top, before any registration.
    base: Keymap,
    pub(super) keymap: Keymap,
    registered: Vec<Registered>,
    problems: Vec<String>,
    /// Set from the source's thread when its settings changed.
    changed: Arc<AtomicBool>,
    watch: Option<Watch>,
}

impl KeyState {
    /// The keymap `source` gives; its conventions, with the reason told, when it cannot be read.
    pub(super) fn load(source: KeySource) -> KeyState {
        let (base, problems) = loaded(&source);
        KeyState {
            source,
            keymap: base.clone(),
            base,
            registered: Vec::new(),
            problems,
            changed: Arc::new(AtomicBool::new(false)),
            watch: None,
        }
    }

    /// Registers `registration` of `app`, and remembers it for the next reload. It replaces the
    /// app's earlier registration, so changed defaults never clash with the app's own old ones;
    /// when it is refused the earlier one stands.
    pub(super) fn register(
        &mut self,
        app: &AppId,
        registration: &Registration,
    ) -> Result<(), Conflict> {
        let wanted = Registered {
            app: app.clone(),
            registration: registration.clone(),
        };
        match self.registered.iter().position(|r| r.app == *app) {
            Some(at) if self.registered[at] == wanted => Ok(()),
            Some(at) => self.replace(at, wanted),
            None => {
                self.keymap.register_with(registration)?;
                self.registered.push(wanted);
                Ok(())
            }
        }
    }

    /// Makes every registration again over the base, with `wanted` in place of the one at `at`.
    fn replace(&mut self, at: usize, wanted: Registered) -> Result<(), Conflict> {
        let mut next = self.registered.clone();
        next[at] = wanted;
        let (keymap, refused) = replayed(&self.base, &next);
        if let Some((_, conflict)) = refused.iter().find(|(app, _)| *app == next[at].app) {
            return Err(conflict.clone());
        }
        self.keymap = keymap;
        self.registered = next;
        self.problems
            .extend(refused.iter().map(|(_, conflict)| conflict.to_string()));
        Ok(())
    }

    /// Reads the source again and makes every registration again over it.
    pub(super) fn reload(&mut self) {
        let (base, mut problems) = loaded(&self.source);
        let (keymap, refused) = replayed(&base, &self.registered);
        problems.extend(refused.iter().map(|(_, conflict)| conflict.to_string()));
        self.base = base;
        self.keymap = keymap;
        self.problems = problems;
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

/// What `source` gives with its overrides on top, and what went wrong reading either.
fn loaded(source: &KeySource) -> (Keymap, Vec<String>) {
    let (keymap, mut problems) = match source.load() {
        Ok(keymap) => (keymap, Vec::new()),
        Err(error) => (
            Keymap::conventional(source.platform()),
            vec![error.to_string()],
        ),
    };
    match source.overrides() {
        Some(overrides) => {
            let (keymap, refused) = keymap.with_overrides(overrides);
            problems.extend(refused.iter().map(ToString::to_string));
            (keymap, problems)
        }
        None => (keymap, problems),
    }
}

/// An app's registration that was refused, and why.
type Refused = (AppId, Conflict);

/// `base` with each registration made over it, and the ones its keymap refused.
fn replayed(base: &Keymap, registered: &[Registered]) -> (Keymap, Vec<Refused>) {
    let mut keymap = base.clone();
    let refused = registered
        .iter()
        .filter_map(|r| {
            let result = keymap.register_with(&r.registration);
            result.err().map(|conflict| (r.app.clone(), conflict))
        })
        .collect();
    (keymap, refused)
}
