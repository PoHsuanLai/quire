//! The type values that belong to the faces (design/02-TYPE.md sections 2 and 5): tracking, weight
//! and size where another set of faces wants different numbers for the same role, so a look that
//! names other faces (the Editorial look Extension) sets these with them.
//!
//! Inter is tracked by its own dynamic metrics (tight at display sizes, zero at body), and the
//! calendar month title needs 600 to hold the line.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One value that follows the faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed)]
#[non_exhaustive]
pub enum VoiceToken {
    /// `--tracking-heading`: `h1`-`h3` (and any heading in the display face).
    #[token(value = "-.02em")]
    TrackingHeading,
    /// `--tracking-lock-clock`: the lock screen's time.
    #[token(value = "-.02em")]
    TrackingLockClock,
    /// `--tracking-lock-date`: the lock screen's date.
    #[token(value = "-.01em")]
    TrackingLockDate,
    /// `--fw-caps`: the calendar month title's weight.
    #[token(name = "fw-caps", value = "600")]
    WeightCaps,
    /// `--tracking-mono`: `.ds-mono`'s tracking.
    #[token(value = "0")]
    TrackingMono,
    /// `--fs-mono`: `.ds-mono`'s size, relative to its parent.
    #[token(value = ".86em")]
    FsMono,
}
