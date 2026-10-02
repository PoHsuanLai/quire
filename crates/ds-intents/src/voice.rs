//! The voice marks (agent-spec voice.md section 3.6): where a summon came from, what a field hears
//! while the person speaks, and which dictation a field is answering. Plain values: the voice
//! wire's own types are converted into these by `docket-ds`, so quire never links an agent crate.

use ds_core::vocab::InputLevel;
use ds_core::word::Word;
use std::fmt;

/// How a summon was started (the view of the agent's summon origin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[word(case = snake)]
pub enum SummonOriginMark {
    /// A key or a click.
    Keyboard,
    /// A held or tapped voice request: the prompt fills with what is heard.
    Voice,
    /// A dictation request: the focused field takes the words.
    Dictation,
}

/// Which dictation a field is answering: the shell mints it when it asks the app to dictate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DictateSerial(pub u64);

/// What a field hears. The words are the person's: `Debug` shows their shape only.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum HeardMark {
    /// The input level (drives the orb only while listening).
    Level(InputLevel),
    /// The unstable tail: shown provisional, not in the model.
    Tail(String),
    /// A stable segment: dictation inserts it.
    Committed(String),
    /// The utterance ended.
    Ended(HeardEndMark),
}

/// How an utterance ended, for a field. `Debug` shows the shape only.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum HeardEndMark {
    /// Send these words.
    Send(String),
    /// Nothing was heard (the field says "Didn't catch that").
    Nothing,
    /// Cancelled or failed: the field restores what it had.
    Cancelled,
}

impl fmt::Debug for HeardMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeardMark::Level(level) => write!(f, "Level({})", level.0),
            HeardMark::Tail(text) => write!(f, "Tail(<{} bytes>)", text.len()),
            HeardMark::Committed(text) => write!(f, "Committed(<{} bytes>)", text.len()),
            HeardMark::Ended(end) => write!(f, "Ended({end:?})"),
        }
    }
}

impl fmt::Debug for HeardEndMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeardEndMark::Send(text) => write!(f, "Send(<{} bytes>)", text.len()),
            HeardEndMark::Nothing => f.write_str("Nothing"),
            HeardEndMark::Cancelled => f.write_str("Cancelled"),
        }
    }
}
