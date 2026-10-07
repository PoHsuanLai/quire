//! What a component asks of the host's frame phase: the step the host runs once per frame, after
//! layout, with the document free. A component never reads or writes the document while it
//! renders (the renderer holds it, so the access answers busy and the caller waits a frame);
//! instead it asks the phase to publish what it observed into a signal when that changes, and to
//! apply a write it queued. The write lands in that frame's document and shows on the next.

use crate::host::position::{TextPosition, TextRange};
use dioxus::prelude::{CopyValue, Signal};
use ds_core::geometry::scroll::Scroll;
use ds_core::geometry::units::{Px, Rect};

/// What the phase publishes about an element, into the signal the component owns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Observe {
    /// The element's border box in logical pixels, after each layout that moved or resized it.
    Rect(Signal<Option<Rect>>),
    /// The element's own scroll state (offset, viewport, content), after each layout or scroll
    /// that changed it, programmatic scrolls included.
    Scroll(Signal<Option<Scroll>>),
    /// The caret's box in an edit surface, relative to the surface's border box. Unlike the
    /// others it is also published *before* a frame paints when `at` moved (see
    /// [`CaretWatch`]), so the caret is drawn in the frame that draws the text it follows.
    Caret(CaretWatch),
    /// The boxes a selection covers in an edit surface, relative to the surface's border box;
    /// published before a frame paints when the range moved, like [`Observe::Caret`].
    Selection(SelectionWatch),
}

/// What the phase publishes for an edit surface's selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionWatch {
    /// The selection, which its owner keeps up to date as it renders (read by the phase each
    /// step, as [`CaretWatch::at`] is).
    pub range: CopyValue<Option<TextRange>>,
    /// The boxes the range covers (one per line of text, one per whole atom); empty while
    /// `range` is `None`, collapsed, or names nothing in the surface.
    pub into: Signal<Vec<Rect>>,
}

/// What the phase publishes for an edit surface's caret.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretWatch {
    /// Where the caret is, which its owner keeps up to date as it renders: the phase reads it
    /// each step, so it never waits for a signal's own wake-up.
    pub at: CopyValue<Option<TextPosition>>,
    /// The caret's box (`--caret-w` wide from the insertion point, its line's height), `None`
    /// while `at` is `None` or names nothing in the surface.
    pub into: Signal<Option<Rect>>,
}

/// A registration with the phase. The phase publishes until it is dropped.
pub struct Watch(Option<Box<dyn FnOnce()>>);

impl Watch {
    /// A registration that runs `forget` when dropped, to end the publishing.
    pub fn new(forget: impl FnOnce() + 'static) -> Self {
        Watch(Some(Box::new(forget)))
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        if let Some(forget) = self.0.take() {
            forget();
        }
    }
}

impl std::fmt::Debug for Watch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Watch")
    }
}

/// What the host answered to an [`Observe`] request.
#[derive(Debug)]
pub enum Observed {
    /// The phase publishes to the signal until the watch is dropped.
    Watching(Watch),
    /// The host runs no phase (a server render, a document nothing drives): the component reads
    /// through the host's other calls instead.
    Unsupported,
}

/// A write a component asks the phase to apply to an element. Writes to one element queue as one:
/// the last replaces the ones before it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhaseWrite {
    /// Scroll the element's own content to this offset, kept inside its scrollable range, with
    /// no animation.
    ScrollTo(Px),
}

/// What the host answered to a [`PhaseWrite`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Queued {
    /// The phase applies it in the frame's next step.
    Yes,
    /// The host runs no phase, or the element is not its own: nothing was queued.
    Unsupported,
}
