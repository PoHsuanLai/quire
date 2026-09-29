//! The host seam an [`EditSurface`](crate::EditSurface) reads geometry and IME through. `ds`
//! stays renderer-free: `ds_native::edit::EDIT` fills it in on Blitz (provided by `launch`, the
//! harness and `ds_native::edit::provide`). Without a host every read answers
//! [`Probe::Unknown`], and the surface still delivers keys, text and clipboard shortcuts.

use crate::core::geometry::units::{Point, Rect};
use crate::host::captured::CapturedPointer;
use crate::host::ime::{ImeEvent, ImeListener, ImeSwitch};
use crate::host::pasted::Pasted;
use crate::host::position::{TextPosition, TextRange};
use crate::host::probe::Probe;
use dioxus::prelude::{EventHandler, MountedData};

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
