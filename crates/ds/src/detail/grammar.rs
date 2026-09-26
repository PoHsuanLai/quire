//! The timing tokens the grammar plays its moments with (design/26-DETAILS.md sections 3.1 and
//! 3.4, and the exits design/05 sections 8 and 10 give Dismiss), as data: what the lint's
//! `Rule::OffGrammarTiming` checks a component's `animation` and `transition` against.

use crate::tokens::{DurationToken, EasingToken};

/// Durations a moment may play for. Left out on purpose: the loops and ambient life
/// (`--t-ambient`, `--t-spin`, `--t-float`, `--t-awake`, `--t-drift`), the orphaned boat
/// (`--t-sail`, `--t-boat-return`), the holds that are not motion (`--t-send-ring`, `--t-flash`)
/// and the Rust-only repaint floor (`--t-count-step`).
pub const GRAMMAR_DURATIONS: [DurationToken; 18] = [
    DurationToken::Tap,
    DurationToken::Quick,
    DurationToken::Move,
    DurationToken::Big,
    DurationToken::BigHeavy,
    DurationToken::Spark,
    DurationToken::Curl,
    DurationToken::CurlHeavy,
    DurationToken::CrumpleHeavy,
    DurationToken::Send,
    DurationToken::HcOut,
    DurationToken::Scene,
    DurationToken::Shake,
    DurationToken::Park,
    DurationToken::Nudge,
    DurationToken::ShakeLong,
    DurationToken::Sweep,
    DurationToken::PendingStep,
];

/// Easings a moment may play along: every easing token names one (the spring only on contact,
/// which the Rust side enforces through `Contact`; the stylesheet cannot see a touch).
pub const GRAMMAR_EASINGS: [EasingToken; 6] = EasingToken::ALL;

/// Whether `token` is a duration the grammar plays.
pub fn is_grammar_duration(token: DurationToken) -> bool {
    GRAMMAR_DURATIONS.contains(&token)
}

/// Whether `token` is an easing the grammar plays along.
pub fn is_grammar_easing(token: EasingToken) -> bool {
    GRAMMAR_EASINGS.contains(&token)
}
