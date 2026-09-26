//! Keeping the command palette's selected stop in view (sill Q340): whoever moved the selection
//! (a key the palette read, a key the caller claimed, the caller's own `selected`, a new query's
//! reset), the list scrolls the least that shows the selected row, cell or header action
//! (`geometry::reveal`), with no animation of its own. A stop the pointer selected is under the
//! pointer already and is left where it is: scrolling it would slide the next row under the
//! pointer and select that one in turn.

use crate::components::palette_rows::Revision;
use crate::geometry::MountedRef;
use crate::geometry::reveal::reveal;
use crate::task::spawn_in;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;

/// What was brought into view last: the stop, its element and the results it was among.
type Shown = (usize, MountedRef, Revision);

/// The list, its stops' elements by stop number, and what was last brought into view.
#[derive(Clone, Copy)]
pub(crate) struct Reveal {
    list: CopyValue<Option<MountedRef>>,
    stops: CopyValue<Vec<Option<MountedRef>>>,
    /// The selection this render asked to show, kept for a list or stop that mounts later.
    asked: CopyValue<Option<(usize, Revision)>>,
    shown: CopyValue<Option<Shown>>,
    task: CopyValue<Option<Task>>,
    scope: ScopeId,
}

/// The palette's reveal book.
pub(crate) fn use_reveal() -> Reveal {
    Reveal {
        list: use_hook(|| CopyValue::new(None)),
        stops: use_hook(|| CopyValue::new(Vec::new())),
        asked: use_hook(|| CopyValue::new(None)),
        shown: use_hook(|| CopyValue::new(None)),
        task: use_hook(|| CopyValue::new(None)),
        scope: use_hook(current_scope_id),
    }
}

impl Reveal {
    /// The list (the scroller) mounted.
    pub(crate) fn list_mounted(self, element: MountedRef) {
        let mut list = self.list;
        list.set(Some(element));
        self.retry();
    }

    /// Stop `index`'s element mounted.
    pub(crate) fn stop_mounted(self, index: usize, element: MountedRef) {
        let mut stops = self.stops;
        stops.with_mut(|stops| {
            if stops.len() <= index {
                stops.resize(index + 1, None);
            }
            stops[index] = Some(element);
        });
        self.retry();
    }

    /// Show stop `current` among the results of `revision`, unless it is what was shown last.
    pub(crate) fn follow(self, current: Option<usize>, revision: Revision) {
        let mut asked = self.asked;
        asked.set(current.map(|index| (index, revision)));
        self.retry();
    }

    /// The pointer selected stop `index`: it counts as shown, so nothing scrolls under it.
    pub(crate) fn pointed(self, index: usize, revision: Revision) {
        let Some(element) = self.element(index) else {
            return;
        };
        let mut shown = self.shown;
        shown.set(Some((index, element, revision)));
    }

    /// Forget what was shown, dropping a scroll still waiting: a palette shown again shows its
    /// selected stop afresh.
    pub(crate) fn forget(self) {
        let (mut shown, mut task) = (self.shown, self.task);
        if let Some(stale) = task.take() {
            stale.cancel();
        }
        shown.set(None);
    }

    fn element(self, index: usize) -> Option<MountedRef> {
        self.stops.peek().get(index).cloned().flatten()
    }

    /// Show the asked stop once the list and it are mounted, if it is not what was shown last.
    fn retry(self) {
        let Some((index, revision)) = *self.asked.peek() else {
            return;
        };
        let (Some(list), Some(element)) = (self.list.peek().clone(), self.element(index)) else {
            return;
        };
        let next = Some((index, element.clone(), revision));
        let mut shown = self.shown;
        if *shown.peek() == next {
            return;
        }
        shown.set(next);
        let mut task = self.task;
        if let Some(stale) = task.take() {
            stale.cancel();
        }
        task.set(Some(spawn_in(self.scope, async move {
            let _ = reveal(&list.0, &element.0).await;
        })));
    }
}
