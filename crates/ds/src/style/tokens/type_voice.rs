//! The type values that follow the typeface (design/02-TYPE.md sections 2 and 5): tracking,
//! weight and size where Inter and the editorial faces want different numbers for the same role.
//!
//! Sizes stay per role in both typefaces except the eyebrow's; what moves is tracking (Inter is
//! tracked by its own dynamic metrics: tight at display sizes, zero at body, and far less than a
//! monospace face wants in caps) and the caps labels' weight (a monospace label reads at 400;
//! Inter caps at that size need 600 to hold the line).

use crate::style::tokens::token::Token;
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
    /// `--tracking-caps`: a tracked uppercase label (eyebrow, section header, menu group).
    #[token(system = ".06em", editorial = ".14em")]
    TrackingCaps,
    /// `--tracking-caps-narrow`: the narrower caps labels (group header, calendar month).
    #[token(system = ".06em", editorial = ".12em")]
    TrackingCapsNarrow,
    /// `--fw-caps`: a tracked uppercase label's weight.
    #[token(name = "fw-caps", system = "600", editorial = "400")]
    WeightCaps,
    /// `--fs-caps`: the eyebrow's size.
    #[token(system = "var(--fs-micro)", editorial = "var(--fs-eyebrow)")]
    FsCaps,
    /// `--tracking-mono`: `.ds-mono`'s tracking.
    #[token(system = "0", editorial = "-.02em")]
    TrackingMono,
    /// `--fs-mono`: `.ds-mono`'s size, relative to its parent.
    #[token(system = ".86em", editorial = ".78em")]
    FsMono,
}
