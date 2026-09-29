//! How the send pill wears a failed send (design/04-COMPONENTS.md section 31, C's outbox).

use crate::core::word::Word;

/// How a send is going, as the pill wears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SendMood {
    /// Nothing wrong: counting, sending or sent.
    #[default]
    Calm,
    /// It will not go: the pill turns to `--danger`.
    Fatal,
}

impl SendMood {
    /// `data-mood`: written only when something is wrong, so a calm pill's markup is as it was.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != SendMood::Calm).then(|| self.slug())
    }
}
