//! A proposed replacement of text, its phases and what moves them.

use crate::components::companion::answer::footer::CardFooter;
use crate::stack::toast_hub::UndoToken;

/// Where a replacement stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplacePhase {
    /// Proposed; nothing in the field has changed. Waits for Apply.
    Proposed,
    /// Applied; the token undoes it.
    Applied {
        /// The undo that puts the original back.
        undo: UndoToken,
    },
    /// Applied, then undone.
    Undone,
    /// Refused.
    Discarded,
}

/// A replacement the companion proposes for text in the person's field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceProposal {
    /// The text as it is.
    pub original: String,
    /// The text it would become.
    pub proposed: String,
    /// Where it stands.
    pub phase: ReplacePhase,
    /// The footer every card has.
    pub footer: CardFooter,
}

/// What moves a replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceIn {
    /// The person applied it.
    Apply,
    /// The person discarded it.
    Discard,
    /// The field changed; here is the undo.
    Applied(UndoToken),
    /// The person undid it.
    Undo,
    /// The undo went through.
    Undone,
    /// The undo window closed.
    Expired,
}

/// What a replacement asks of its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceOut {
    /// Apply it to the field.
    Apply,
    /// Drop it.
    Discard,
    /// Undo it with this token.
    Undo(UndoToken),
}
