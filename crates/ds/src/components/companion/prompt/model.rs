//! The prompt's state, its inputs and its outputs.

use ds_core::word::Word;
use ds_intents::ContextChip;

/// A field's caret and selection, in characters: the anchor and where the caret is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CaretSpan {
    /// Where the selection began.
    pub anchor: usize,
    /// Where the caret is; equal to the anchor for a bare caret.
    pub focus: usize,
}

/// What the field gives back when prompt mode ends: the query verbatim and its caret.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Kept {
    /// The text the field held when the summon came.
    pub query: String,
    /// Where the caret was.
    pub caret: CaretSpan,
}

/// Where a prompt stands once it is on, `data-phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PromptPhase {
    /// The person is typing it.
    Composing,
    /// It has been sent; the answer is awaited.
    Sent,
    /// An answer has come.
    Answered,
}

/// A field's prompt state: off, or on with the query it kept, the chips it carries and the text
/// typed so far.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PromptState {
    /// An ordinary field.
    #[default]
    Off,
    /// The field is the companion's prompt.
    On {
        /// What to give back when it ends.
        kept: Kept,
        /// The context the prompt carries.
        chips: Vec<ContextChip>,
        /// The prompt typed so far.
        text: String,
        /// Where the prompt stands.
        phase: PromptPhase,
    },
}

/// What moves the prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptIn {
    /// A summon reached the field, with the query it holds and the context frozen at that moment.
    Summon {
        /// The query, as it was.
        kept: Kept,
        /// The context the prompt will carry.
        chips: Vec<ContextChip>,
    },
    /// The person typed.
    Typed(String),
    /// The person sent the prompt.
    Send,
    /// An answer arrived.
    Answered,
    /// Escape.
    Escape,
    /// The summon came again while the prompt was open.
    SummonAgain,
    /// The person dropped the chip at this index.
    ChipRemoved(usize),
}

/// What the prompt asks of its field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOut {
    /// The field enters prompt mode.
    Enter,
    /// Send this prompt with these chips.
    Submit {
        /// The prompt text.
        prompt: String,
        /// The chips that stayed.
        chips: Vec<ContextChip>,
    },
    /// Give the field its query back.
    Restore(Kept),
    /// Cancel the ask in flight.
    CancelAsk,
}
