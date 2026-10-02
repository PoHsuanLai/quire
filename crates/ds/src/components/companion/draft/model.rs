//! A draft reply.

use crate::components::companion::answer::action::CardAction;
use crate::components::companion::answer::footer::CardFooter;
use crate::components::content::avatar::AvatarFace;

/// A person on a message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonLine {
    /// Their name.
    pub name: String,
    /// Their address.
    pub address: String,
    /// Their avatar.
    pub avatar: AvatarFace,
}

/// A reply the companion drafted, for the person to edit and send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftReply {
    /// Who it goes to.
    pub to: Vec<PersonLine>,
    /// Its subject.
    pub subject: String,
    /// The message it answers, quoted.
    pub quoted: Option<String>,
    /// The body, editable in place.
    pub body: String,
    /// What the person can do: Send (outbound), Save draft (undoable write), Open in Mail.
    pub actions: Vec<CardAction>,
    /// The footer every card has.
    pub footer: CardFooter,
}
