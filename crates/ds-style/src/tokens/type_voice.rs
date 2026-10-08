//! The type values that follow the typeface (design/02-TYPE.md sections 2 and 5): tracking,
//! weight and size where Inter and the editorial faces want different numbers for the same role.
//!
//! Sizes stay per role in both typefaces except `.ds-mono`'s; what moves is tracking (Inter is
//! tracked by its own dynamic metrics: tight at display sizes, zero at body) and the calendar
//! month title's weight (a monospace title reads at 400; Inter at that size needs 600 to hold the
//! line).

use crate::tokens::token::Token;
use ds_core::word::Word;

/// One typeface-dependent value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed)]
pub enum VoiceToken {
    /// `--tracking-heading`: `h1`-`h3` (and any heading in the display face).
    #[token(system = "-.02em", editorial = "-.015em")]
    TrackingHeading,
    /// `--tracking-lock-clock`: the lock screen's time.
    #[token(system = "-.02em", editorial = "-.035em")]
    TrackingLockClock,
    /// `--tracking-lock-date`: the lock screen's date.
    #[token(system = "-.01em", editorial = ".01em")]
    TrackingLockDate,
    /// `--fw-caps`: the calendar month title's weight.
    #[token(name = "fw-caps", system = "600", editorial = "400")]
    WeightCaps,
    /// `--tracking-mono`: `.ds-mono`'s tracking.
    #[token(system = "0", editorial = "-.02em")]
    TrackingMono,
    /// `--fs-mono`: `.ds-mono`'s size, relative to its parent.
    #[token(system = ".86em", editorial = ".78em")]
    FsMono,
    /// `--lh-term`: a terminal row's height as a multiple of its font size; a grid of box
    /// drawing wants rows close enough to join.
    #[token(name = "lh-term", system = "1.2", editorial = "1.2")]
    LineHeightTerminal,
}
