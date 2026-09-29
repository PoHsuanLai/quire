//! Editing through an app's own editor core (mailo's composer, FINDINGS "Edit surface"): the
//! vocabulary [`EditSurface`](crate::EditSurface) speaks, and the host seam it reads geometry
//! and IME through.
//!
//! The app keeps its document model and draws its own caret and selection. The surface only
//! owns the focus, turns keys, IME composition and clipboard shortcuts into [`EditInput`]s,
//! resolves a pointer to a [`TextPosition`] in the app's own markup, and answers caret and
//! selection rects through the host (`ds_native::edit`). Positions name the app's elements by
//! their `data-edit-node` value, so the app never sees a renderer's node ids.

pub(crate) mod clicks;
pub(crate) mod composition;
pub(crate) mod handle;
pub(crate) mod host;
pub(crate) mod input;
pub(crate) mod keys;
pub(crate) mod pointer;
