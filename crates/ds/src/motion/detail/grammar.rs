//! The timing tokens the grammar plays its moments with (design/26-DETAILS.md sections 3.1 and
//! 3.4, and the exits design/05 sections 8 and 10 give Dismiss), as data: what the lint's
//! `Rule::OffGrammarTiming` checks a component's `animation` and `transition` against.

use crate::core::word::Word;
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};

/// Durations a moment may play for. Left out on purpose: the loops and ambient life
/// (`--t-ambient`, `--t-spin`, `--t-float`, `--t-awake`), the orphaned boat (`--t-sail`,
/// `--t-boat-return`), the holds that are not motion (`--t-send-ring`, `--t-flash`), the Rust-only
/// repaint floor (`--t-count-step`) and the battery ring's Rust-driven fill (`--t-fill`, which
/// predates `--t-sweep`; design/05 section 4.12).
pub const GRAMMAR_DURATIONS: [DurationToken; 15] = [
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
    DurationToken::Shake,
    DurationToken::Park,
    DurationToken::Nudge,
    DurationToken::Sweep,
    DurationToken::PendingStep,
];

/// Easings a moment may play along: every easing token names one (the spring only on contact,
/// which the Rust side enforces through `Contact`; the stylesheet cannot see a touch).
pub const GRAMMAR_EASINGS: &[EasingToken] = EasingToken::ALL;
