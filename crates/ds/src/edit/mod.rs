//! Editing through an app's own editor core (mailo's composer, FINDINGS "Edit surface"): the
//! vocabulary [`EditSurface`](crate::EditSurface) speaks, and the host seam it reads geometry
//! and IME through.
//!
//! The app keeps its document model and draws its own caret and selection. The surface only
//! owns the focus, turns keys, IME composition and clipboard shortcuts into [`EditInput`]s,
//! resolves a pointer to a [`TextPosition`] in the app's own markup, and answers caret and
//! selection rects through the host (`ds_native::edit`). Positions name the app's elements by
//! their `data-edit-node` value, so the app never sees a renderer's node ids.

pub mod clicks;
pub mod composition;
pub mod handle;
pub mod host;
pub mod input;
pub mod keys;
pub mod pointer;
pub mod position;

pub use clicks::Clicks;
pub use handle::{EditHandle, use_edit_handle};
pub use host::{HostEdit, ImeEvent, ImeListener, ImeSwitch, Probe};
pub use input::{Composition, EditInput, KeyInput, Pasted, PreeditCursor};
pub use pointer::{CapturedPointer, EditFocus, EditPointer, Extend, PointerPhase};
pub use position::{
    EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditNode, TextOffset, TextPosition, TextRange,
};
