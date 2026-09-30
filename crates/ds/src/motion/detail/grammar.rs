//! The timing tokens the grammar plays its moments with (design/26-DETAILS.md sections 3.1 and
//! 3.4, and the exits design/05 sections 8 and 10 give Dismiss), as data: what the lint's
//! `Rule::OffGrammarTiming` checks a component's `animation` and `transition` against.

use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
use ds_core::word::Word;

/// Durations a moment may play for. Left out on purpose: the loops (`--t-awake`), the spinner's
/// step (`--t-spin-step`), the holds that are not motion (`--t-send-ring`) and the idle overlay's
/// Rust-driven fade (`--t-idle-dim`).
pub const GRAMMAR_DURATIONS: [DurationToken; 4] = [
    DurationToken::Quick,
    DurationToken::Move,
    DurationToken::Big,
    DurationToken::Shake,
];

/// Easings a moment may play along: every easing token names one (the spring only on contact,
/// which the Rust side enforces through `Contact`; the stylesheet cannot see a touch).
pub const GRAMMAR_EASINGS: &[EasingToken] = EasingToken::ALL;
