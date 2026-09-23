//! Dragging without HTML5 drag events: a Rust tracker owns the pointer, the 8 px Manhattan
//! threshold and the drop-target hit test (design/06-INTERACTIONS.md section 6).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::geometry::units::{Point, Px, Rect};
use dioxus::prelude::*;

/// Where a drag is.
#[derive(Debug, Clone, PartialEq)]
pub enum DragPhase<K> {
    /// No pointer down.
    Idle,
    /// Pointer down on `key` at `from`; nothing visible until it moves past the threshold.
    Pending {
        /// What is being dragged.
        key: K,
        /// Where the pointer went down.
        from: Point,
    },
    /// Dragging `key`; the ghost is at `at`, over `target` if any.
    Live {
        /// What is being dragged.
        key: K,
        /// The pointer.
        at: Point,
        /// The index of the registered drop target under the pointer.
        target: Option<usize>,
    },
}

/// The pure drag machine.
#[derive(Debug, Clone, PartialEq)]
pub struct Drag<K> {
    phase: DragPhase<K>,
    threshold: Px,
    targets: Vec<Rect>,
}

impl<K: Clone + PartialEq> Drag<K> {
    /// A drag that goes live after `threshold` of Manhattan movement.
    pub fn new(threshold: Px) -> Self {
        todo!()
    }

    /// Pointer down on `key`.
    pub fn down(self, key: K, at: Point) -> Self {
        todo!()
    }

    /// Pointer moved.
    pub fn moved(self, at: Point) -> Self {
        todo!()
    }

    /// Pointer released: the target it was dropped on, if the drag was live over one.
    pub fn up(self) -> (Self, Option<(K, usize)>) {
        todo!()
    }

    /// Replace the drop-target rects the hit test runs against.
    pub fn with_targets(self, targets: Vec<Rect>) -> Self {
        todo!()
    }

    /// Where the drag is.
    pub fn phase(&self) -> &DragPhase<K> {
        &self.phase
    }
}

/// A live drag machine in a signal.
#[derive(Debug, PartialEq)]
pub struct DragTracker<K: 'static> {
    drag: Signal<Drag<K>>,
}

impl<K: 'static> Clone for DragTracker<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for DragTracker<K> {}

impl<K: Clone + PartialEq + 'static> DragTracker<K> {
    /// Where the drag is.
    pub fn phase(&self) -> DragPhase<K> {
        todo!()
    }

    /// Feed a pointer-down on `key`.
    pub fn down(&self, key: K, at: Point) {
        todo!()
    }

    /// Feed a pointer move.
    pub fn moved(&self, at: Point) {
        todo!()
    }

    /// Feed a pointer-up; returns the drop, if any.
    pub fn up(&self) -> Option<(K, usize)> {
        todo!()
    }
}

/// A drag tracker with a Manhattan `threshold` (8 px in the prototypes).
pub fn use_drag<K: Clone + PartialEq + 'static>(threshold: Px) -> DragTracker<K> {
    todo!()
}
