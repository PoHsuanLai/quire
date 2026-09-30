//! A masked field's own caret and selection over its dots.
//!
//! Blitz's text editor ignores `font-family` and `letter-spacing`, so the hidden
//! text of a `Password` or `Secret` field is measured in another face than its Inter dots tracked
//! .1em, and Blitz's caret drifted off the last dot as the text grew (two dots short at eleven).
//! Where the host reads the field's selection ([`CaretHost::selection`](crate::CaretHost::selection), ds-native), the field hides
//! Blitz's caret (`caret-color: transparent`) and the mask draws its own: a 1.5 px bar the height
//! of the line, as Blitz's is, placed between the dots at the caret's character. A selected
//! range is a highlight over its dots and shows no caret. Without the host the renderer's
//! caret stays.

use crate::components::fields::text_input_kind::MASK_DOT;
use crate::host::caret::FieldSelection;
use crate::host::document::DocumentHost;
use dioxus::core::ScopeId;
use dioxus::prelude::*;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_style::task::{spawn_in, try_set_if_changed};
use std::rc::Rc;

/// How many frames a read waits out a busy document before it gives up.
const READ_ATTEMPTS: u8 = 4;

/// Who draws a masked field's caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaretOwner {
    /// The renderer: no host reads the selection, or the field shows no dots.
    Renderer,
    /// The mask, over its dots: the input's own caret is transparent (`data-caret=drawn`).
    Mask,
}

impl CaretOwner {
    /// The input's `data-caret`.
    pub(crate) fn data_caret(self) -> Option<&'static str> {
        match self {
            CaretOwner::Renderer => None,
            CaretOwner::Mask => Some("drawn"),
        }
    }
}

/// Whether the mask shows its caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaretMark {
    /// A bar between `before` and `after`.
    Shown,
    /// None: the field is not focused, a range is selected, or the renderer draws it.
    Hidden,
}

/// The mask's dots cut at the selection: `before`, the `selected` range, the caret, `after`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MaskParts {
    pub before: String,
    pub selected: String,
    pub caret: CaretMark,
    pub after: String,
}

fn dots(count: usize) -> String {
    std::iter::repeat_n(MASK_DOT, count).collect()
}

impl MaskParts {
    /// `count` dots cut at `selection` (clamped to them), with the caret the mask draws.
    pub(crate) fn cut(count: usize, selection: FieldSelection, owner: CaretOwner) -> Self {
        let whole = MaskParts {
            before: dots(count),
            selected: String::new(),
            caret: CaretMark::Hidden,
            after: String::new(),
        };
        match (owner, selection) {
            (CaretOwner::Mask, FieldSelection::Focused { anchor, focus }) => {
                let (low, high) = (anchor.min(focus).min(count), anchor.max(focus).min(count));
                let caret = if low == high {
                    CaretMark::Shown
                } else {
                    CaretMark::Hidden
                };
                MaskParts {
                    before: dots(low),
                    selected: dots(high - low),
                    caret,
                    after: dots(count - high),
                }
            }
            (CaretOwner::Renderer, _)
            | (_, FieldSelection::Unfocused | FieldSelection::Unknown) => whole,
        }
    }
}

/// The field's last read selection, the host that reads it and the element it reads.
#[derive(Clone, Copy)]
pub(crate) struct MaskCaret {
    selection: Signal<FieldSelection>,
    element: CopyValue<Option<Rc<MountedData>>>,
    /// The document's host, when one was provided: it reads the selection.
    host: CopyValue<Option<Rc<dyn DocumentHost>>>,
    owner: ScopeId,
}

impl MaskCaret {
    /// The hook: one per field, kept across renders.
    pub(crate) fn use_new() -> Self {
        MaskCaret {
            selection: use_signal(|| FieldSelection::Unfocused),
            element: use_hook(|| CopyValue::new(None)),
            host: use_hook(|| CopyValue::new(try_consume_context::<Rc<dyn DocumentHost>>())),
            owner: use_hook(dioxus::core::current_scope_id),
        }
    }

    /// Who draws the caret of a field with `count` masked characters.
    pub(crate) fn owner(&self, count: usize) -> CaretOwner {
        match (self.host.peek().as_ref(), count) {
            (Some(_), 1..) => CaretOwner::Mask,
            (None, _) | (_, 0) => CaretOwner::Renderer,
        }
    }

    /// The selection last read.
    pub(crate) fn selection(&self) -> FieldSelection {
        (self.selection)()
    }

    /// Keep the field's element for the reads.
    pub(crate) fn mounted(&self, event: &MountedEvent) {
        let mut element = self.element;
        element.set(Some(event.data()));
    }

    /// Read the selection again once the event that may have moved it is done: Blitz moves the
    /// caret after the handlers ran, and a task runs after that.
    pub(crate) fn refresh(&self) {
        let (Some(host), Some(element)) = (self.host.peek().clone(), self.element.peek().clone())
        else {
            return;
        };
        let selection = self.selection;
        spawn_in(self.owner, async move {
            for _ in 0..READ_ATTEMPTS {
                match host.caret().selection(&element) {
                    FieldSelection::Unknown => sleep(FRAME_SLACK).await,
                    seen => {
                        let _ = try_set_if_changed(selection, seen);
                        return;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{CaretMark, CaretOwner, MaskParts};
    use crate::host::caret::FieldSelection;

    fn at(anchor: usize, focus: usize) -> FieldSelection {
        FieldSelection::Focused { anchor, focus }
    }

    /// Four dots cut: how many before, selected and after, and whether the caret shows.
    fn cut(selection: FieldSelection, owner: CaretOwner) -> (usize, usize, CaretMark, usize) {
        let parts = MaskParts::cut(4, selection, owner);
        (
            parts.before.chars().count(),
            parts.selected.chars().count(),
            parts.caret,
            parts.after.chars().count(),
        )
    }

    #[test]
    fn the_mask_cuts_its_dots_at_the_caret_and_the_selection() {
        use CaretMark::{Hidden, Shown};
        use CaretOwner::{Mask, Renderer};
        assert_eq!(cut(at(4, 4), Mask), (4, 0, Shown, 0), "at the end");
        assert_eq!(cut(at(1, 1), Mask), (1, 0, Shown, 3), "after the first");
        assert_eq!(cut(at(0, 0), Mask), (0, 0, Shown, 4), "at the start");
        assert_eq!(cut(at(3, 1), Mask), (1, 2, Hidden, 1), "a range, backwards");
        assert_eq!(cut(at(9, 9), Mask), (4, 0, Shown, 0), "clamped to the dots");
        assert_eq!(
            cut(FieldSelection::Unfocused, Mask),
            (4, 0, Hidden, 0),
            "unfocused"
        );
        assert_eq!(cut(at(1, 1), Renderer), (4, 0, Hidden, 0), "no host");
    }
}
