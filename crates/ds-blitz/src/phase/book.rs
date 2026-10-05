//! What the phase holds between frames: the signals it publishes to, and the writes waiting to be
//! applied (data only; `run` does the work).

use blitz_dom::NodeId;
use dioxus::prelude::Signal;
use ds::host::phase::{Observe, PhaseWrite};
use ds::prelude::Rect;

/// One registration, so a component's drop can end it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct WatchId(u64);

/// A rect the phase publishes for one node.
pub(super) struct RectWatch {
    pub(super) id: WatchId,
    pub(super) node: NodeId,
    pub(super) sink: Signal<Option<Rect>>,
    /// What was last published: only a different read is published again.
    pub(super) last: Option<Rect>,
}

/// A write waiting for its frame.
#[derive(Clone, Copy)]
pub(super) struct Waiting {
    pub(super) node: NodeId,
    pub(super) write: PhaseWrite,
}

#[derive(Default)]
pub(super) struct Book {
    next: u64,
    pub(super) rects: Vec<RectWatch>,
    pub(super) writes: Vec<Waiting>,
}

impl Book {
    /// Register `what` for `node`.
    pub(super) fn watch(&mut self, node: NodeId, what: Observe) -> WatchId {
        let id = WatchId(self.next);
        self.next += 1;
        match what {
            Observe::Rect(sink) => self.rects.push(RectWatch {
                id,
                node,
                sink,
                last: None,
            }),
        }
        id
    }

    /// End a registration.
    pub(super) fn forget(&mut self, id: WatchId) {
        self.rects.retain(|watch| watch.id != id);
    }

    /// Queue `write` for `node`, replacing one already waiting for it.
    pub(super) fn queue(&mut self, node: NodeId, write: PhaseWrite) {
        self.writes.retain(|waiting| waiting.node != node);
        self.writes.push(Waiting { node, write });
    }
}

#[cfg(test)]
mod tests {
    use super::Book;
    use blitz_dom::NodeId;
    use ds::host::phase::PhaseWrite;
    use ds::prelude::Px;

    #[test]
    fn the_last_write_to_a_node_replaces_the_ones_before_it() {
        let mut book = Book::default();
        book.queue(NodeId::from_u64(3), PhaseWrite::ScrollTo(Px(10.0)));
        book.queue(NodeId::from_u64(4), PhaseWrite::ScrollTo(Px(99.0)));
        book.queue(NodeId::from_u64(3), PhaseWrite::ScrollTo(Px(40.0)));
        let queued: Vec<(u64, PhaseWrite)> = book
            .writes
            .iter()
            .map(|waiting| (waiting.node.as_u64(), waiting.write))
            .collect();
        assert_eq!(
            queued,
            [
                (4, PhaseWrite::ScrollTo(Px(99.0))),
                (3, PhaseWrite::ScrollTo(Px(40.0)))
            ]
        );
    }
}
