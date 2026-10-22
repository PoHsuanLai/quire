//! The pointer's side of dragging rows of a `VirtualTable`: the press that may begin a drag, the
//! target under the pointer as it moves, the scrolling at the table's edges, and the drop.
//!
//! The state is the table's, not the rows': a row unmounts when it scrolls out of the window, and
//! a drag that scrolls the list far enough would lose its source if the row held it. The press
//! names its row (an index) and the rest of the drag is arithmetic on the pointer's height in the
//! content (`drag_model`), so a drag over rows that are not mounted works the same. Like the
//! rubber band, the list scrolls a little on each pointer move made near an edge, and a pointer
//! held still near the edge does not scroll on its own, as quire keeps no timer for a gesture.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::components::lists::marquee::hook::Marquee;
use crate::components::lists::marquee::model::edge_scroll;
use crate::components::lists::virtual_table::drag_model::{
    RowDrop, RowDropKind, RowDropTarget, RowGaps, allowed, dragged_rows, drop_target, operation_of,
};
use crate::host::event_point::pointer_point;
use chordkit::Platform;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::selection::RowSelection;
use ds_motion::drag::DRAG_THRESHOLD;

/// The height of the line drawn in a gap, and so how far above the gap's edge it starts.
const LINE: f32 = 3.0;

/// Where a drag is.
#[derive(Debug, Clone, PartialEq)]
enum Phase {
    Idle,
    /// The press is down on `row` at `from` and has not moved past the threshold.
    Pending { row: usize, from: Point },
    /// Carrying `rows`, over `target` if the pointer is over a place that takes them.
    Live {
        rows: Vec<usize>,
        target: Option<RowDropTarget>,
    },
}

/// What the table tells its drag each render: the rows and what they are as places to drop.
#[derive(Clone, PartialEq)]
pub(crate) struct RowDragSpec {
    pub(crate) platform: Platform,
    pub(crate) pitch: Px,
    pub(crate) rows: usize,
    /// The owner's selection, if it keeps one: a drag from a selected row carries it whole.
    pub(crate) selection: Option<RowSelection>,
    pub(crate) kind: Option<Callback<usize, RowDropKind>>,
    pub(crate) gaps: RowGaps,
    pub(crate) ondrop: EventHandler<RowDrop>,
}

/// A table's row drag: made by [`use_row_drag`], handed to the rows' presses, the table's pointer
/// handlers and [`RowDropMark`].
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RowDrag {
    phase: Signal<Phase>,
    spec: CopyValue<Option<RowDragSpec>>,
    marquee: Marquee,
    scroller: ScrollerRef,
}

/// The row drag of the calling table, over the box its rubber band is measured in.
pub(crate) fn use_row_drag(marquee: Marquee, scroller: ScrollerRef) -> RowDrag {
    RowDrag {
        phase: use_signal(|| Phase::Idle),
        spec: use_hook(|| CopyValue::new(None)),
        marquee,
        scroller,
    }
}

impl RowDrag {
    /// What the drag works from this render; `None` when the owner does not listen for drops, and
    /// then no press begins a drag.
    pub(crate) fn aim(self, spec: Option<RowDragSpec>) {
        let mut cell = self.spec;
        cell.set(spec);
    }

    /// A press on `row`: with the primary button, a drag may begin from it.
    pub(crate) fn pressed(self, row: usize, event: &PointerEvent) {
        if self.spec.peek().is_none() || event.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        let mut phase = self.phase;
        phase.set(Phase::Pending {
            row,
            from: pointer_point(event),
        });
    }

    /// The pointer moved. Past the threshold the drag is live; while it is, the list scrolls if
    /// the pointer is near an edge and the target is the place under the pointer.
    pub(crate) fn moved(self, event: &PointerEvent) {
        let Some(spec) = self.spec.peek().clone() else {
            return;
        };
        let at = pointer_point(event);
        let rows = match self.phase.peek().clone() {
            Phase::Idle => return,
            Phase::Pending { row, from } => {
                if travelled(from, at) < DRAG_THRESHOLD.0 {
                    return;
                }
                dragged_rows(row, spec.selection.as_ref())
            }
            Phase::Live { rows, .. } => rows,
        };
        let target = self.target_at(at, &spec, &rows);
        let next = Phase::Live { rows, target };
        if *self.phase.peek() != next {
            let mut phase = self.phase;
            phase.set(next);
        }
    }

    /// The pointer was released: rows carried over a target are dropped on it, with the
    /// operation the modifiers held at the release ask for.
    pub(crate) fn released(self, event: &PointerEvent) {
        let phase = self.phase.peek().clone();
        if phase == Phase::Idle {
            return;
        }
        self.cancel();
        let spec = self.spec.peek().clone();
        if let (
            Phase::Live {
                rows,
                target: Some(target),
            },
            Some(spec),
        ) = (phase, spec)
        {
            spec.ondrop.call(RowDrop {
                rows,
                target,
                operation: operation_of(spec.platform, event.modifiers()),
            });
        }
    }

    /// The drag is called off (Escape, the pointer leaving the table): nothing is dropped.
    pub(crate) fn cancel(self) {
        if *self.phase.peek() != Phase::Idle {
            let mut phase = self.phase;
            phase.set(Phase::Idle);
        }
    }

    /// Where the live drag would land now; `None` with no drag or over no place that takes it.
    fn landing(self) -> Option<RowDropTarget> {
        match self.phase.read().clone() {
            Phase::Live { target, .. } => target,
            Phase::Idle | Phase::Pending { .. } => None,
        }
    }

    /// The place under the pointer at `at` that takes `rows`, scrolling the list when the pointer
    /// is near its top or bottom edge.
    fn target_at(self, at: Point, spec: &RowDragSpec, rows: &[usize]) -> Option<RowDropTarget> {
        let (view, content) = self.marquee.place(at)?;
        let delta = edge_scroll(at.y.0 - view.top().0, view.size.height.0);
        if delta.abs() > 0.0 {
            self.scroller.by(Px(delta));
        }
        let kind = |row: usize| spec.kind.as_ref().map_or(RowDropKind::Item, |kind| kind.call(row));
        let found = drop_target(content.y.0, spec.pitch.0, spec.rows, spec.gaps, kind)?;
        allowed(found, rows)
    }
}

/// How far the pointer went from `from` to `to`, straight-line.
fn travelled(from: Point, to: Point) -> f32 {
    (to.x.0 - from.x.0).hypot(to.y.0 - from.y.0)
}

/// The drop indicator while rows are carried over a place that takes them, nothing otherwise: the
/// container row's highlight (`data-drop=onto`) or a line in the gap (`data-drop=between`). Like
/// the rubber band it is a zero-height layer at the top of the scroller's content, so it scrolls
/// with the rows; it reads the drag itself, so the pointer moving redraws this and not the table.
#[component]
pub(crate) fn RowDropMark(drag: RowDrag, pitch: Px) -> Element {
    let Some(target) = drag.landing() else {
        return rsx! {};
    };
    let (kind, top, height) = match target {
        RowDropTarget::Onto(row) => ("onto", row as f32 * pitch.0, pitch.0),
        RowDropTarget::Between(gap) => ("between", gap as f32 * pitch.0 - LINE / 2.0, LINE),
    };
    rsx! {
        div { class: "ds-vtable-drop-layer",
            div {
                class: "ds-vtable-drop",
                "aria-hidden": "true",
                "data-drop": kind,
                style: "top:{top}px;height:{height}px",
            }
        }
    }
}
