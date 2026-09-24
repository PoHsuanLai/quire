//! The command palette's row rects: the element each line is drawn as, and the selected row's
//! rect reported to a caller that anchors to it, an actions menu opened on the selected row
//! (sill FINDINGS Q41).
//!
//! The read goes through the measurer a frame after layout (spike S9) and is tried again while
//! the row has no area yet: a palette on a surface that is mapped again reads its first row
//! before the surface's first layout, and a 0 x 0 rect is never reported (sill FINDINGS Q60).
//! The row is measured again whenever the selection, the element under it, or the results
//! change; a read still waiting for layout is dropped when a newer one starts.

use crate::geometry::measure::laid_out_rect;
use crate::geometry::{MountedRef, Rect};
use crate::task::spawn_in;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;

/// The selection as a choice number and the line that choice is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedLine {
    /// The choice (items and submenu parents, counted in order).
    pub choice: usize,
    /// The line it is drawn on (headers counted).
    pub line: usize,
}

/// How many times the palette's results (its groups and its field's tokens, which move the list)
/// have changed since it mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Revision(u32);

/// The revision of `value`: bumped on every render that sees it differ from the last one.
pub(crate) fn use_revision<V: Clone + PartialEq + 'static>(value: &V) -> Revision {
    let mut seen = use_hook(|| CopyValue::new((value.clone(), Revision::default())));
    let revision = {
        let (last, revision) = &*seen.peek();
        if last == value {
            return *revision;
        }
        *revision
    };
    let next = Revision(revision.0.wrapping_add(1));
    seen.set((value.clone(), next));
    next
}

/// What a report was made for: the selection, the element under it, and the results.
type Measured = (SelectedLine, MountedRef, Revision);

/// The rows the palette has mounted, by line, and what it last measured and reported.
#[derive(Clone, Copy)]
pub(crate) struct RowRects {
    rows: CopyValue<Vec<Option<MountedRef>>>,
    measured: CopyValue<Option<Measured>>,
    reported: CopyValue<Option<Rect>>,
    reading: CopyValue<Option<Task>>,
    on_rect: Option<EventHandler<Rect>>,
    scope: ScopeId,
}

/// The palette's row book, reporting to `on_rect` when there is one.
pub(crate) fn use_row_rects(on_rect: Option<EventHandler<Rect>>) -> RowRects {
    RowRects {
        rows: use_hook(|| CopyValue::new(Vec::new())),
        measured: use_hook(|| CopyValue::new(None)),
        reported: use_hook(|| CopyValue::new(None)),
        reading: use_hook(|| CopyValue::new(None)),
        on_rect,
        scope: use_hook(current_scope_id),
    }
}

impl RowRects {
    /// The element on `line` mounted; report it if it is the selected row.
    pub(crate) fn mounted(
        self,
        line: usize,
        element: MountedRef,
        selected: Option<SelectedLine>,
        revision: Revision,
    ) {
        let mut rows = self.rows;
        rows.with_mut(|rows| {
            if rows.len() <= line {
                rows.resize(line + 1, None);
            }
            rows[line] = Some(element);
        });
        self.follow(selected, revision);
    }

    /// Measure the selected row again when the selection, the element under it, or the
    /// results changed since the last measurement, and report its rect once it has an area.
    pub(crate) fn follow(self, selected: Option<SelectedLine>, revision: Revision) {
        let (Some(on_rect), Some(selected)) = (self.on_rect, selected) else {
            return;
        };
        let Some(element) = self.rows.peek().get(selected.line).cloned().flatten() else {
            return;
        };
        let next = Some((selected, element.clone(), revision));
        let mut measured = self.measured;
        if *measured.peek() == next {
            return;
        }
        measured.set(next);
        let mut reading = self.reading;
        if let Some(stale) = reading.take() {
            stale.cancel();
        }
        let reported = self.reported;
        reading.set(Some(spawn_in(self.scope, async move {
            if let Some(rect) = laid_out_rect(&element.0).await {
                report(reported, rect, on_rect);
            }
        })));
    }

    /// Forget what was measured and reported, dropping a read still waiting: a palette shown
    /// again reports its selected row afresh, even where the row has not moved.
    pub(crate) fn forget(self) {
        let (mut measured, mut reported, mut reading) =
            (self.measured, self.reported, self.reading);
        if let Some(stale) = reading.take() {
            stale.cancel();
        }
        measured.set(None);
        reported.set(None);
    }
}

/// Hand `rect` to `on_rect` unless it is the rect reported last.
fn report(reported: CopyValue<Option<Rect>>, rect: Rect, on_rect: EventHandler<Rect>) {
    let mut reported = reported;
    let Ok(mut last) = reported.try_write() else {
        return;
    };
    if *last == Some(rect) {
        return;
    }
    *last = Some(rect);
    drop(last);
    on_rect.call(rect);
}
