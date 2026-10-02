//! The activity strip's data.

use crate::components::companion::mark::AppMark;
use crate::stack::toast_hub::UndoToken;
use ds_core::vocab::{ActorMark, Fraction, Tally};

/// Where an activity entry stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityState {
    /// Running, and how far when it knows.
    Running(Option<Fraction>),
    /// Waiting for the person.
    NeedsYou,
    /// Done.
    Done,
    /// Failed.
    Failed,
    /// Undone.
    Undone,
    /// Its undo window has closed.
    Expired,
}

/// What the person can do about an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoOffer {
    /// Undo this one.
    Undo,
    /// Undo every step of the run; how many there are.
    UndoAll(Tally),
    /// Nothing to undo.
    Not,
}

/// One thing that was done, by whom, in which app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEntry {
    /// Its journal id: the undo token it hands back.
    pub key: UndoToken,
    /// What was done, in words.
    pub title: String,
    /// Who did it.
    pub actor: ActorMark,
    /// The app it was done in.
    pub app: Option<AppMark>,
    /// When, in words.
    pub when: String,
    /// Where it stands.
    pub state: ActivityState,
    /// What the person can do about it.
    pub undo: UndoOffer,
}
