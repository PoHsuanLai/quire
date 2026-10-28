//! Which document asked for something: the app's own (no [`FrameId`]), or a frame inside it. An
//! `<iframe>`'s sub-document is a separate Blitz document with its own id, so everything it does (a fetch, a
//! link click) can be told apart from the app's by that id alone.

/// One frame's document: an `<iframe>`'s sub-document, named by Blitz's document id. A frame
/// that is reloaded (a new `srcdoc`) is a new document with a new id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(usize);

impl FrameId {
    /// The frame whose Blitz document id is `document`.
    pub fn of(document: usize) -> Self {
        FrameId(document)
    }

    /// A number naming this frame's document for as long as the process runs: a stable key for
    /// the app's own maps (mailo's per-frame state), never reused by a later document.
    pub fn index(self) -> usize {
        self.0
    }
}
