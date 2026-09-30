//! Files dragged into the window from outside (a file manager, a desktop): which of the app's
//! drop targets the pointer is over, and the paths it lets go of there (mailo: attachments
//! dropped onto the composer).
//!
//! `ds` stays renderer-free: the host hears the platform's drag, turns it into
//! [`FileDragInput`]s and feeds them to a [`FileDropBoard`], which it provides as root context, and
//! the board finds targets through the host's own hit test in the document. ds-blitz does both, for `launch`'s window and for the
//! harness (`Harness::file_drag`). Without a host a target simply never lights.
//!
//! Paths only: a drag that carries no `file:` URI (a link or text dragged out of a browser) is
//! refused, and no target lights for it.

pub mod board;
pub mod drag;
pub mod hook;
pub(crate) mod track;
