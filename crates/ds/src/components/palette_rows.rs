//! The command palette's row rects: the element each line is drawn as, and the selected row's
//! rect reported to a caller that anchors to it, an actions menu opened on the selected row
//! (sill FINDINGS Q41). The read goes through the measurer a frame after layout (spike S9).

use crate::geometry::measure::client_rect;
use crate::geometry::{MountedRef, Rect};
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;

/// The selection as a choice number and the line that choice is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedLine {
    /// The choice (items and submenu parents, counted in order).
    pub choice: usize,
    /// The line it is drawn on (headers counted).
    pub line: usize,
}

/// The rows the palette has mounted, by line, and what it last reported.
#[derive(Clone, Copy)]
pub(crate) struct RowRects {
    rows: CopyValue<Vec<Option<MountedRef>>>,
    reported: CopyValue<Option<(SelectedLine, MountedRef)>>,
    on_rect: Option<EventHandler<Rect>>,
}

/// The palette's row book, reporting to `on_rect` when there is one.
pub(crate) fn use_row_rects(on_rect: Option<EventHandler<Rect>>) -> RowRects {
    RowRects {
        rows: use_hook(|| CopyValue::new(Vec::new())),
        reported: use_hook(|| CopyValue::new(None)),
        on_rect,
    }
}

impl RowRects {
    /// The element on `line` mounted; report it if it is the selected row.
    pub(crate) fn mounted(self, line: usize, element: MountedRef, selected: Option<SelectedLine>) {
        let mut rows = self.rows;
        rows.with_mut(|rows| {
            if rows.len() <= line {
                rows.resize(line + 1, None);
            }
            rows[line] = Some(element);
        });
        self.follow(selected);
    }

    /// Report the selected row's rect when the selection, or the element under it, changed
    /// since the last report.
    pub(crate) fn follow(self, selected: Option<SelectedLine>) {
        let (Some(on_rect), Some(selected)) = (self.on_rect, selected) else {
            return;
        };
        let Some(element) = self.rows.peek().get(selected.line).cloned().flatten() else {
            return;
        };
        let next = Some((selected, element.clone()));
        let mut reported = self.reported;
        if *reported.peek() == next {
            return;
        }
        reported.set(next);
        spawn(async move {
            sleep(FRAME_SLACK).await;
            if let Some(rect) = client_rect(&element.0).await {
                on_rect.call(rect);
            }
        });
    }
}
