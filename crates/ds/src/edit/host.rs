//! The host seam an [`EditSurface`](crate::EditSurface) reads geometry and IME through. `ds`
//! stays renderer-free: `ds_native::edit::EDIT` fills it in on Blitz (provided by `launch`, the
//! harness and `ds_native::edit::provide`). Without a host every read answers
//! [`Probe::Unknown`], and the surface still delivers keys, text and clipboard shortcuts.

use crate::edit::input::Pasted;
use crate::edit::pointer::CapturedPointer;
use crate::edit::position::{TextPosition, TextRange};
use crate::geometry::{Point, Rect};
use dioxus::prelude::{EventHandler, MountedData};

/// One read or write through the host.
#[derive(Debug, Clone, PartialEq)]
pub enum Probe<T> {
    /// The answer.
    Found(T),
    /// The document is busy (rendering); ask again next frame.
    Busy,
    /// The host cannot answer: no host, the surface is not its node or is gone, nothing
    /// addressable is there, or the document has not been laid out yet.
    Unknown,
}

impl<T> Probe<T> {
    /// The answer, if there was one.
    pub fn found(self) -> Option<T> {
        match self {
            Probe::Found(value) => Some(value),
            Probe::Busy | Probe::Unknown => None,
        }
    }
}

/// What the IME tells the focused surface, as the host receives it (winit's `Ime`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// The IME attached to the window.
    Enabled,
    /// The preedit is now `text` (empty: cleared), with the IME's cursor as UTF-8 byte offsets.
    Preedit {
        /// The text being composed.
        text: String,
        /// The IME's cursor or highlight in it.
        cursor: Option<(usize, usize)>,
    },
    /// Insert `text`.
    Commit(String),
    /// The IME detached.
    Disabled,
}

/// Whether the window's IME is on for the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImeSwitch {
    /// On: the surface has the keyboard.
    On,
    /// Off: it lost it.
    Off,
}

/// A surface's registration for IME events, to cancel as it unmounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImeListener(pub u64);

/// The host's edit operations, provided as root context. Each takes the surface's element, so
/// the host searches only its subtree. Points and rects are the window's logical pixels.
#[derive(Debug, Clone, Copy)]
pub struct HostEdit {
    /// The text position under a point.
    pub hit_test: fn(&MountedData, Point) -> Probe<TextPosition>,
    /// The caret's box at a position: `--caret-w` wide (whole device pixels), the height of its
    /// line, starting at the insertion point.
    pub caret_rect: fn(&MountedData, &TextPosition) -> Probe<Rect>,
    /// The boxes a selection covers, one per line of text and one per whole atom.
    pub selection_rects: fn(&MountedData, &TextRange) -> Probe<Vec<Rect>>,
    /// Turn the window's IME on or off for the surface.
    pub set_ime: fn(&MountedData, ImeSwitch) -> Probe<()>,
    /// Where the IME's candidate window should sit: the caret's rect.
    pub set_ime_cursor_area: fn(&MountedData, Rect) -> Probe<()>,
    /// The clipboard, with its HTML when it has any. Call from a handler.
    pub read_clipboard_html: fn() -> Option<Pasted>,
    /// Deliver the IME's events to `sink` while the surface (or a node inside it) has the focus.
    pub listen: fn(&MountedData, EventHandler<ImeEvent>) -> Probe<ImeListener>,
    /// Stop delivering to a listener.
    pub forget: fn(ImeListener),
    /// Route every pointer move and the release to `sink` until the button comes up, wherever
    /// the pointer is: called at a press on the surface.
    pub capture: fn(&MountedData, EventHandler<CapturedPointer>) -> Probe<()>,
}
