//! The answer a card shows.

use super::action::CardAction;
use super::footer::CardFooter;
use super::refused::RefusedAnswer;
use crate::components::companion::draft::model::DraftReply;
use crate::components::companion::form::model::CompactForm;
use crate::components::companion::plan::model::PlanView;
use crate::components::companion::proposed_event::model::ProposedEvent;
use crate::components::companion::replace::model::ReplaceProposal;
use crate::components::content::text_runs::TextLine;
use ds_core::word::Word;

/// What a card shows: one of the answer shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerView {
    /// Plain text.
    Text(TextAnswer),
    /// A draft reply.
    DraftReply(DraftReply),
    /// An event the companion proposes.
    ProposedEvent(ProposedEvent),
    /// A plan.
    Plan(PlanView),
    /// A change to the text in the person's field.
    Replace(ReplaceProposal),
    /// A form for what the companion still needs.
    Form(CompactForm),
    /// A refusal.
    Refused(RefusedAnswer),
}

/// Whether a text answer is still arriving, `data-streaming`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Streaming {
    /// Text is still coming.
    Arriving,
    /// It is whole.
    Complete,
}

/// A text answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAnswer {
    /// The lines, as the shell displays them: never model markup.
    pub body: Vec<TextLine>,
    /// Whether more is coming.
    pub streaming: Streaming,
    /// What the person can do with it.
    pub actions: Vec<CardAction>,
    /// The footer every card has.
    pub footer: CardFooter,
}
