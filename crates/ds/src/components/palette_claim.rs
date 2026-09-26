//! The palette's key claim (sill Q299): a caller hears each key its field gets before the palette
//! or the field does, with where the caret is, and may take it. A taken key is the caller's
//! alone: the palette does not read it, the field does not type or move its caret (its default
//! is prevented), and `onkey` does not hear it. The launcher takes Space while the person is
//! browsing (to toggle its preview pane) and Right with the caret at the end of the query (to
//! show it); a Space typed while searching is left to the field.

pub use crate::focus::caret::Caret;
use dioxus::prelude::*;

/// A key the palette's field got, before anything acted on it.
#[derive(Debug, Clone)]
pub struct FieldKey {
    /// The key event: its key, code and modifiers.
    pub event: KeyboardEvent,
    /// Where the caret was when it arrived.
    pub caret: Caret,
}

/// The caller's answer for a [`FieldKey`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Claim {
    /// The caller takes it: nothing else acts on it.
    Take,
    /// The palette, the field and `onkey` go on as if the caller had not looked.
    Pass,
}
