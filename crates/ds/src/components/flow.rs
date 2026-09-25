//! Whether a surface that normally floats is drawn in the overlay or in place (mailo gaps 4).
//!
//! A menu or a hover card floats by default: it renders at the end of `.ds`, placed against its
//! anchor, and joins the layer stack where it takes Escape. Some callers want the same rows or
//! the same card inside a container of their own: mailo's sender card lists its actions as a
//! menu inside the card, and a test with no layout asserts a card's content where it put it.
//! One type serves both components, since the choice is the same fact about either. The scrim
//! took it too (mailo gaps 5): a reader peeked inside a pane dims the pane beneath it with an
//! inline scrim, which the overlay's scrim layer would have drawn above the reader.

/// Where a menu, a hover card or a scrim is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Flow {
    /// In the overlay host, placed against its anchor (the default).
    #[default]
    Floating,
    /// Where the caller renders it, in the document's flow: `position:static`, no overlay, no
    /// scrim, no layer on the stack, no focus taken. Written as `data-flow="inline"`.
    Inline,
}

impl Flow {
    /// The `data-flow` word: only an inline surface carries one, so a floating one's markup is
    /// what it was.
    pub(crate) fn attr(self) -> Option<&'static str> {
        match self {
            Flow::Floating => None,
            Flow::Inline => Some("inline"),
        }
    }
}
