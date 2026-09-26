//! The type values that follow the typeface (design/02-TYPE.md sections 2 and 5): tracking,
//! weight and size where Inter and the editorial faces want different numbers for the same role.
//!
//! Sizes stay per role in both typefaces except the eyebrow's; what moves is tracking (Inter is
//! tracked by its own dynamic metrics: tight at display sizes, zero at body, and far less than a
//! monospace face wants in caps) and the caps labels' weight (a monospace label reads at 400;
//! Inter caps at that size need 600 to hold the line).

use super::name::VarName;
use crate::appearance::Typeface;

/// One typeface-dependent value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceToken {
    /// `--tracking-heading`: `h1`-`h3` (and any heading in the display face).
    TrackingHeading,
    /// `--tracking-lock-clock`: the lock screen's time.
    TrackingLockClock,
    /// `--tracking-lock-date`: the lock screen's date.
    TrackingLockDate,
    /// `--tracking-caps`: a tracked uppercase label (eyebrow, section header, menu group).
    TrackingCaps,
    /// `--tracking-caps-narrow`: the narrower caps labels (group header, calendar month).
    TrackingCapsNarrow,
    /// `--fw-caps`: a tracked uppercase label's weight.
    WeightCaps,
    /// `--fs-caps`: the eyebrow's size.
    FsCaps,
    /// `--tracking-mono`: `.ds-mono`'s tracking.
    TrackingMono,
    /// `--fs-mono`: `.ds-mono`'s size, relative to its parent.
    FsMono,
}

impl VoiceToken {
    /// Every token, in the order the stylesheet declares them.
    pub const ALL: [VoiceToken; 9] = [
        VoiceToken::TrackingHeading,
        VoiceToken::TrackingLockClock,
        VoiceToken::TrackingLockDate,
        VoiceToken::TrackingCaps,
        VoiceToken::TrackingCapsNarrow,
        VoiceToken::WeightCaps,
        VoiceToken::FsCaps,
        VoiceToken::TrackingMono,
        VoiceToken::FsMono,
    ];

    /// The custom property.
    pub fn var(self) -> VarName {
        VarName(match self {
            VoiceToken::TrackingHeading => "--tracking-heading",
            VoiceToken::TrackingLockClock => "--tracking-lock-clock",
            VoiceToken::TrackingLockDate => "--tracking-lock-date",
            VoiceToken::TrackingCaps => "--tracking-caps",
            VoiceToken::TrackingCapsNarrow => "--tracking-caps-narrow",
            VoiceToken::WeightCaps => "--fw-caps",
            VoiceToken::FsCaps => "--fs-caps",
            VoiceToken::TrackingMono => "--tracking-mono",
            VoiceToken::FsMono => "--fs-mono",
        })
    }

    /// The value under `typeface`. Editorial's are the values the rules carried before the
    /// typeface existed (design/02-TYPE.md sections 3-5), so mail keeps its look exactly.
    pub fn css(self, typeface: Typeface) -> &'static str {
        match typeface {
            Typeface::System => match self {
                VoiceToken::TrackingHeading | VoiceToken::TrackingLockClock => "-.02em",
                VoiceToken::TrackingLockDate => "-.01em",
                VoiceToken::TrackingCaps | VoiceToken::TrackingCapsNarrow => ".06em",
                VoiceToken::WeightCaps => "600",
                VoiceToken::FsCaps => "var(--fs-micro)",
                VoiceToken::TrackingMono => "0",
                VoiceToken::FsMono => ".86em",
            },
            Typeface::Editorial => match self {
                VoiceToken::TrackingHeading => "-.015em",
                VoiceToken::TrackingLockClock => "-.035em",
                VoiceToken::TrackingLockDate => ".01em",
                VoiceToken::TrackingCaps => ".14em",
                VoiceToken::TrackingCapsNarrow => ".12em",
                VoiceToken::WeightCaps => "400",
                VoiceToken::FsCaps => "var(--fs-eyebrow)",
                VoiceToken::TrackingMono => "-.02em",
                VoiceToken::FsMono => ".78em",
            },
        }
    }
}
