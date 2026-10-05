//! The phase itself: a window's (or the harness's) handle on its queue of writes and its watches,
//! and the step the loop runs.

use super::book::{Book, WatchId};
use super::read::{Moved, apply, border_box};
use crate::node_ref::{DocRef, NodeRef, Written};
use dioxus::core::{Runtime, RuntimeGuard};
use dioxus::prelude::MountedData;
use ds::host::phase::{Observe, Observed, PhaseWrite, Queued, Watch};
use ds::prelude::Rect;
use std::cell::RefCell;
use std::rc::Rc;

/// Whether layout is up to date when the phase runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    /// Right after a frame was resolved: reads are current and writes clamp against the content as
    /// drawn. Writes are applied once and dropped.
    Resolved,
    /// After the document was updated and before it was resolved: only writes run, and they are
    /// kept to run again once layout has caught up.
    Pending,
}

/// What one run of the phase did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ran {
    /// Writes that moved something in the document.
    pub applied: usize,
    /// Signals that were given a changed value.
    pub published: usize,
}

impl Ran {
    /// Whether the run changed the document or a signal, so the frame must be drawn again.
    pub fn changed(self) -> bool {
        self.applied + self.published > 0
    }
}

/// The document the phase works on and the runtime its signals belong to.
#[derive(Clone)]
struct Drive {
    doc: DocRef,
    runtime: Rc<Runtime>,
}

#[derive(Default)]
struct Shared {
    book: RefCell<Book>,
    drive: RefCell<Option<Drive>>,
}

/// One document's phase. Cloned freely: the host that serves components and the loop that runs it
/// hold the same one. Until a document is attached it supports nothing.
#[derive(Clone, Default)]
pub struct Phase {
    shared: Rc<Shared>,
}

impl std::fmt::Debug for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Phase").finish_non_exhaustive()
    }
}

impl Phase {
    /// Work on `doc`, publishing into signals that live in `runtime`.
    pub fn attach(&self, doc: DocRef, runtime: Rc<Runtime>) {
        self.shared.drive.replace(Some(Drive { doc, runtime }));
    }

    /// Publish what `what` names for the element `el`, until the watch is dropped.
    pub(crate) fn observe(&self, el: &MountedData, what: Observe) -> Observed {
        let (Some(_), Some(node)) = (self.shared.drive.borrow().as_ref(), NodeRef::of(el)) else {
            return Observed::Unsupported;
        };
        let id = self.shared.book.borrow_mut().watch(node.node, what);
        let book = Rc::clone(&self.shared);
        Observed::Watching(Watch::new(move || forget(&book, id)))
    }

    /// Queue `write` to the element `el` for the next run.
    pub(crate) fn write(&self, el: &MountedData, write: PhaseWrite) -> Queued {
        let (Some(_), Some(node)) = (self.shared.drive.borrow().as_ref(), NodeRef::of(el)) else {
            return Queued::Unsupported;
        };
        self.shared.book.borrow_mut().queue(node.node, write);
        Queued::Yes
    }

    /// Run one step: apply the queued writes, then (once layout is [`Layout::Resolved`]) publish
    /// every watched value that changed. The document is borrowed only while it is read or
    /// written, and not at all while signals are set. A document that is borrowed elsewhere is
    /// left for the next run.
    pub fn run(&self, layout: Layout) -> Ran {
        let Some(drive) = self.shared.drive.borrow().clone() else {
            return Ran::default();
        };
        let applied = self.apply_writes(&drive, layout);
        let published = match layout {
            Layout::Resolved => self.publish(&drive),
            Layout::Pending => 0,
        };
        Ran { applied, published }
    }

    fn apply_writes(&self, drive: &Drive, layout: Layout) -> usize {
        let writes = {
            let mut book = self.shared.book.borrow_mut();
            match layout {
                Layout::Resolved => std::mem::take(&mut book.writes),
                Layout::Pending => book.writes.clone(),
            }
        };
        if writes.is_empty() {
            return 0;
        }
        let mut moved = Vec::new();
        let written = drive.doc.write(|doc| {
            moved = writes
                .iter()
                .map(|waiting| apply(doc, waiting.node, waiting.write))
                .collect();
        });
        match written {
            Written::Done => moved.iter().filter(|moved| **moved == Moved::Yes).count(),
            Written::Busy => {
                if layout == Layout::Resolved {
                    // Put back what was taken, behind anything queued since.
                    let mut book = self.shared.book.borrow_mut();
                    for waiting in writes {
                        if !book.writes.iter().any(|held| held.node == waiting.node) {
                            book.writes.push(waiting);
                        }
                    }
                }
                0
            }
        }
    }

    fn publish(&self, drive: &Drive) -> usize {
        let wanted: Vec<(WatchId, Option<Rect>)> = {
            let book = self.shared.book.borrow();
            let Some(read) = drive.doc.read(|doc| {
                book.rects
                    .iter()
                    .map(|watch| (watch.id, border_box(doc, watch.node)))
                    .collect()
            }) else {
                return 0;
            };
            read
        };
        let changed: Vec<_> = {
            let mut book = self.shared.book.borrow_mut();
            wanted
                .into_iter()
                .filter_map(|(id, read)| {
                    let watch = book.rects.iter_mut().find(|watch| watch.id == id)?;
                    (watch.last != read).then(|| {
                        watch.last = read;
                        (watch.sink, read)
                    })
                })
                .collect()
        };
        let _in_runtime = RuntimeGuard::new(Rc::clone(&drive.runtime));
        // A sink whose component has gone is skipped: its watch is dropped with it.
        changed
            .into_iter()
            .filter(|(sink, read)| ds::style::task::try_set(*sink, *read).is_ok())
            .count()
    }
}

fn forget(shared: &Shared, id: WatchId) {
    shared.book.borrow_mut().forget(id);
}
