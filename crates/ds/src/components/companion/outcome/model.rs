//! How a finished run ended, as a mark the person has not looked at yet.

use ds_core::word::Word;

/// How a run ended, for the unseen-outcome mark. The consumer passes it while the person has
/// not looked at the result and passes `None` once they have; the mark keeps no memory of its
/// own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Outcome {
    /// It finished what it was asked to do.
    Done,
    /// It could not finish.
    Failed,
}
