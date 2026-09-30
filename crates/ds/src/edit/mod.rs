//! Editing through an app's own editor core (mailo's composer, FINDINGS "Edit surface"): the
//! vocabulary `EditSurface` speaks, and the host seam it reads geometry
//! and IME through.
//!
//! The app keeps its document model and draws its own caret and selection. The surface only
//! owns the focus, turns keys, IME composition and clipboard shortcuts into [`EditInput`]s,
//! resolves a pointer to a [`TextPosition`] in the app's own markup, and answers caret and
//! selection rects through the host (`ds_blitz::edit`). Positions name the app's elements by
//! their `data-edit-node` value, so the app never sees a renderer's node ids.

pub mod clicks;
pub(crate) mod composition;
pub mod handle;
pub mod input;
pub(crate) mod keys;
pub mod pointer;
