//! The parts of the document seam. Each is a cohesive handful of reads and writes; a host
//! implements them all (`DocumentHost`), and every method answers `Busy` or `Unknown` rather than
//! panicking when the renderer holds the document or the element is not its own.

use crate::host::captured::CapturedPointer;
use crate::host::caret::{Caret, CaretOwed, FieldSelection, InitialCaret};
use crate::host::drop_hit::DropHit;
use crate::host::fallback::Fallback;
use crate::host::focused::Focused;
use crate::host::found::{Found, SameNode};
use crate::host::hand_back::HandBack;
use crate::host::ime::{ImeEvent, ImeListener, ImeSwitch};
use crate::host::measure::Measured;
use crate::host::pasted::Pasted;
use crate::host::phase::{Observe, Observed, PhaseWrite, Queued};
use crate::host::position::{TextPosition, TextRange};
use crate::host::probe::Probe;
use crate::host::reveal::Scrolled;
use dioxus::prelude::{EventHandler, MountedData};
use ds_core::geometry::units::{Point, Rect};
use std::rc::Rc;

/// Moving the keyboard focus into and out of an element, and where it goes when its holder leaves.
pub trait FocusHost {
    /// Give `el` the keyboard.
    fn focus(&self, el: &MountedData) -> Focused;
    /// Take the keyboard from `el`: [`Focused::Done`] when it had it and no longer has it.
    fn blur(&self, el: &MountedData) -> Focused;
    /// Select all of `el`'s text, once the caret is in it.
    fn select(&self, el: &MountedData) -> Focused;
    /// Give `el` the keyboard and put its caret at `at` in the same write, so a key typed the
    /// moment the focus lands never meets a caret placed afterwards from a stale view of the
    /// text. A host that cannot place a caret from here selects all for
    /// [`InitialCaret::SelectAll`] and only focuses for the rest; a field that already has the
    /// keyboard keeps its caret.
    fn focus_placing(&self, el: &MountedData, at: InitialCaret) -> Focused {
        match (self.focus(el), at) {
            (Focused::Done, InitialCaret::SelectAll) => self.select(el),
            (focused, _) => focused,
        }
    }
    /// Where the keyboard goes when a surface that took it is removed.
    fn hand_back(&self) -> &HandBack;
}

/// A text field's caret and selection.
pub trait CaretHost {
    /// Where the caret is in the field `el`.
    fn caret(&self, el: &MountedData) -> Caret;
    /// Put the caret of the focused field `el` at `at`.
    fn place_caret(&self, el: &MountedData, at: InitialCaret) -> Focused;
    /// The field's selection, in characters.
    fn selection(&self, el: &MountedData) -> FieldSelection;
    /// Whether the focused field `el`, not laid out yet, still owes its caret. A host that cannot
    /// tell owes nothing.
    fn caret_owed(&self, el: &MountedData) -> CaretOwed {
        let _ = el;
        CaretOwed::No
    }
}

/// Where things are in the document.
pub trait GeometryHost {
    /// `el`'s border box, in logical pixels.
    fn measure(&self, el: &MountedData) -> Measured;
    /// Scroll `list`'s own content the least that shows `item`, with no animation.
    fn reveal(&self, list: &MountedData, item: &MountedData) -> Scrolled;
    /// The first element matching a CSS selector.
    fn find(&self, selector: &str) -> Found;
    /// Whether `a` and `b` are the same node.
    fn same(&self, a: &MountedData, b: &MountedData) -> SameNode;
    /// Publish what the frame phase sees of `el` into a signal, whenever it changes, until the
    /// returned watch is dropped. A host with no phase answers [`Observed::Unsupported`].
    fn observe(&self, _el: &MountedData, _what: Observe) -> Observed {
        Observed::Unsupported
    }
    /// Queue a write to `el` for the frame phase to apply. A host with no phase answers
    /// [`Queued::Unsupported`].
    fn write(&self, _el: &MountedData, _write: PhaseWrite) -> Queued {
        Queued::Unsupported
    }
}

/// Where the keyboard goes after a click on nothing focusable.
pub trait ClickFocusHost {
    /// Read the click from the document through `root`, before the renderer's default action.
    fn fallback(&self, root: &MountedData) -> Fallback;
    /// Give the ancestor `fallback` found the keyboard once the click has run, if the click left
    /// the focus nowhere.
    fn restore(&self, ancestor: &MountedData) -> Focused;
    /// Give the pressed control, or its nearest focusable ancestor, the keyboard now: a click a
    /// control kept with its default prevented.
    fn press(&self, root: &MountedData) -> Focused;
}

/// What an edit surface reads geometry, the clipboard and the pointer through. Each takes the
/// surface's element, so the host searches only its subtree; points and rects are the window's
/// logical pixels.
pub trait EditHost {
    /// The text position under a point.
    fn hit_test(&self, surface: &MountedData, at: Point) -> Probe<TextPosition>;
    /// The caret's box at a position.
    fn caret_rect(&self, surface: &MountedData, at: &TextPosition) -> Probe<Rect>;
    /// The boxes a selection covers, one per line of text and one per whole atom.
    fn selection_rects(&self, surface: &MountedData, range: &TextRange) -> Probe<Vec<Rect>>;
    /// The clipboard, with its HTML when it has any. Call from a handler.
    fn paste(&self) -> Option<Pasted>;
    /// Route every pointer move and the release to `sink` until the button comes up, wherever the
    /// pointer is: called at a press on the surface.
    fn capture(&self, surface: &MountedData, sink: EventHandler<CapturedPointer>) -> Probe<()>;
    /// The window's IME.
    fn ime(&self) -> &dyn ImeHost;
}

/// The window's IME, as an edit surface uses it.
pub trait ImeHost {
    /// Turn the IME on or off for the surface.
    fn switch(&self, surface: &MountedData, to: ImeSwitch) -> Probe<()>;
    /// Where the IME's candidate window should sit: the caret's rect.
    fn cursor_area(&self, surface: &MountedData, area: Rect) -> Probe<()>;
    /// Deliver the IME's events to `sink` while the surface (or a node inside it) has the focus.
    fn listen(&self, surface: &MountedData, sink: EventHandler<ImeEvent>) -> Probe<ImeListener>;
    /// Stop delivering to a listener.
    fn forget(&self, listener: ImeListener);
}

/// The host's hit test for a file drag.
pub trait FileDropHost {
    /// The innermost of `targets` under `at`, or the one whose element is an ancestor of what is.
    fn hit(&self, targets: &[Rc<MountedData>], at: Point) -> DropHit;
}
