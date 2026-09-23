//! Dragging without HTML5 drag events: a Rust tracker owns the pointer, the 8 px Manhattan
//! threshold and the drop-target hit test (design/06-INTERACTIONS.md section 6).

use crate::components::vocab::Fraction;
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
        Drag {
            phase: DragPhase::Idle,
            threshold,
            targets: Vec::new(),
        }
    }

    /// Pointer down on `key`.
    pub fn down(self, key: K, at: Point) -> Self {
        Drag {
            phase: DragPhase::Pending { key, from: at },
            ..self
        }
    }

    /// Pointer moved.
    pub fn moved(self, at: Point) -> Self {
        let phase = match self.phase {
            DragPhase::Idle => DragPhase::Idle,
            DragPhase::Pending { key, from } if manhattan(from, at) < self.threshold.0 => {
                DragPhase::Pending { key, from }
            }
            DragPhase::Pending { key, .. } | DragPhase::Live { key, .. } => DragPhase::Live {
                key,
                at,
                target: hit(&self.targets, at),
            },
        };
        Drag { phase, ..self }
    }

    /// Pointer released: the target it was dropped on, if the drag was live over one.
    pub fn up(self) -> (Self, Option<(K, usize)>) {
        let dropped = match self.phase {
            DragPhase::Live {
                key,
                target: Some(index),
                ..
            } => Some((key, index)),
            _ => None,
        };
        (
            Drag {
                phase: DragPhase::Idle,
                ..self
            },
            dropped,
        )
    }

    /// Replace the drop-target rects the hit test runs against.
    pub fn with_targets(self, targets: Vec<Rect>) -> Self {
        let phase = match self.phase {
            DragPhase::Live { key, at, .. } => DragPhase::Live {
                key,
                at,
                target: hit(&targets, at),
            },
            other => other,
        };
        Drag {
            phase,
            targets,
            ..self
        }
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
        self.drag.read().phase().clone()
    }

    /// Feed a pointer-down on `key`.
    pub fn down(&self, key: K, at: Point) {
        self.step(|drag| drag.down(key, at));
    }

    /// Feed a pointer move.
    pub fn moved(&self, at: Point) {
        self.step(|drag| drag.moved(at));
    }

    /// Feed a pointer-up; returns the drop, if any.
    pub fn up(&self) -> Option<(K, usize)> {
        let (next, dropped) = self.drag.peek().clone().up();
        let mut drag = self.drag;
        drag.set(next);
        dropped
    }

    /// Replace the drop-target rects, measured by the consumer (`use_rect`); the index of a
    /// drop is the index into this list.
    pub fn set_targets(&self, targets: Vec<Rect>) {
        self.step(|drag| drag.with_targets(targets));
    }

    fn step(&self, f: impl FnOnce(Drag<K>) -> Drag<K>) {
        let next = f(self.drag.peek().clone());
        let mut drag = self.drag;
        drag.set(next);
    }
}

/// A drag tracker with a Manhattan `threshold` (8 px in the prototypes).
pub fn use_drag<K: Clone + PartialEq + 'static>(threshold: Px) -> DragTracker<K> {
    DragTracker {
        drag: use_signal(|| Drag::new(threshold)),
    }
}

/// `|dx| + |dy|` between two points.
fn manhattan(from: Point, to: Point) -> f32 {
    (to.x.0 - from.x.0).abs() + (to.y.0 - from.y.0).abs()
}

/// The first target containing `at`: left and top edges inside, right and bottom outside.
fn hit(targets: &[Rect], at: Point) -> Option<usize> {
    targets.iter().position(|rect| {
        at.x.0 >= rect.left().0
            && at.x.0 < rect.right().0
            && at.y.0 >= rect.top().0
            && at.y.0 < rect.bottom().0
    })
}

/// Where a pointer at `x` sits along `track`, in thousandths, clamped to the track and
/// rounded to the nearest multiple of `step` (a zero step is continuous): the Slider's value
/// while its thumb is dragged (design/04-COMPONENTS.md section 5).
pub fn fraction_along(track: Rect, x: Px, step: Fraction) -> Fraction {
    let width = track.size.width.0;
    let raw = if width > 0.0 {
        ((x.0 - track.left().0) / width * 1000.0).clamp(0.0, 1000.0)
    } else {
        0.0
    };
    let permille = match step.0 {
        0 => raw.round(),
        n => ((raw / f32::from(n)).round() * f32::from(n)).min(1000.0),
    };
    // In 0..=1000 after the clamp, so the cast cannot truncate.
    Fraction(permille as u16)
}
