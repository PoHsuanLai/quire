//! Which document asked for something: the app's own, or a frame inside it. An `<iframe>`'s
//! sub-document is a separate Blitz document with its own id, so everything it does (a fetch, a
//! link click) can be told apart from the app's by that id alone.

/// One frame's document: an `<iframe>`'s sub-document, named by Blitz's document id. A frame
/// that is reloaded (a new `srcdoc`) is a new document with a new id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(usize);

impl FrameId {
    /// The frame whose Blitz document id is `document`.
    pub(crate) fn of(document: usize) -> Self {
        FrameId(document)
    }
}

/// The document a request came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestOrigin {
    /// The app's own document: its markup is the app's.
    Top,
    /// A frame's sub-document: its markup is someone else's (a mail body), so nothing is served
    /// to it unless the app says so for that request.
    Frame(FrameId),
}
