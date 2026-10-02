//! A proposed event and the day it would go in.

use crate::components::companion::answer::action::CardAction;
use crate::components::companion::answer::footer::CardFooter;
use crate::components::companion::draft::model::PersonLine;
use ds_core::word::Word;

/// A stretch of a day, in minutes from midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinuteSpan {
    /// Where it starts.
    pub from: u16,
    /// Where it ends.
    pub to: u16,
}

/// What a slot of the day strip is, `data-kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum BusyKind {
    /// Already in the calendar.
    Existing,
    /// The proposed event.
    Proposed,
    /// Overlaps something already there.
    Conflict,
}

/// One slot of the day strip.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DaySlot {
    /// When.
    pub span: MinuteSpan,
    /// What it is.
    pub kind: BusyKind,
    /// What it says.
    pub label: String,
}

/// An event the companion proposes, drawn on a strip of the day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedEvent {
    /// Its title.
    pub title: String,
    /// When, in words.
    pub when: String,
    /// Where.
    pub place: Option<String>,
    /// Who is invited.
    pub people: Vec<PersonLine>,
    /// The day it lands in.
    pub day: Vec<DaySlot>,
    /// What the person can do.
    pub actions: Vec<CardAction>,
    /// The footer every card has.
    pub footer: CardFooter,
}
