//! The summon seam's plain values: a summon and how a field answers it.

use ds_core::word::Word;

/// Which summon an ask or an answer belongs to: sill mints it when it calls the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SummonSerial(pub u64);

/// How an app answered a summon: the view of the agent's summon answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[word(case = snake)]
pub enum SummonAnswerMark {
    /// The focused field became the prompt.
    TookField,
    /// A prompt anchored at the selection opened.
    TookAnchored,
    /// The prompt was open and the summon closed it, restoring the query.
    Restored,
    /// Nothing here takes a prompt; the launcher should.
    Declined,
}
