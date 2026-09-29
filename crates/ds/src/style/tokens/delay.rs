//! Delays and holds: the two CSS delays (`--d-fly`, `--d-heal`) and the Rust-only timer lengths
//! (design/05-MOTION.md sections 3.4 and 7.2).
//!
//! These measure intent or reading time, not motion, so they do not scale with the level
//! (design/05-MOTION.md open decision 2, proposed: hover-intent delays unchanged under Reduced).

use super::token::{CssValue, Token, TokenScope};
use crate::core::word::Word;
use crate::style::appearance::motion::MotionLevel;
use std::time::Duration;

/// A delay the stylesheet reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "d-", kind = fixed, css = delay_css)]
pub enum StyleDelay {
    /// `--d-fly` 350 ms: before a strip button's preview label shows.
    Fly,
    /// 18 ms per row: the heal ripple (also a CSS delay step).
    #[token(name = "heal")]
    HealStep,
}

impl StyleDelay {
    /// How long it lasts at `level`.
    pub fn delay(self, level: MotionLevel) -> Duration {
        Duration::from_millis(match self {
            StyleDelay::Fly => 350,
            StyleDelay::HealStep if level == MotionLevel::Reduced => 0,
            StyleDelay::HealStep => 18,
        })
    }
}

/// A delay as the stylesheet writes it, at the scope's motion level.
fn delay_css(token: StyleDelay, scope: TokenScope) -> CssValue {
    CssValue::computed(format!("{}ms", token.delay(scope.motion).as_millis()))
}

/// A timer length only Rust reads: intent and reading time, never scaled by the level
/// (design/30-CATALOGUE.md section 1.2). Each is the default of a settings key of the same
/// name where one exists; the settings override it, this table is where the default lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum DelayToken {
    /// 1000 ms: a tooltip opens after the pointer rests (the `Tip` hover profile).
    TipOpen,
    /// 500 ms: a hover card opens after the pointer rests (the `Card` profile).
    CardOpen,
    /// 150 ms: a hover card closes after the pointer leaves.
    CardClose,
    /// 100 ms: a dock label opens after the pointer rests (the `Label` profile).
    LabelOpen,
    /// 400 ms: after one hover UI closes, the next opens with no delay within this window.
    HoverWarm,
    /// 500 ms: a press held without moving is a long press.
    LongPress,
    /// 200 ms: a submenu opens after the pointer rests on its parent row.
    SubmenuOpen,
    /// 300 ms: how long a menu tracks the pointer inside the safe triangle toward a submenu.
    TriangleTimeout,
    /// 500 ms: the system double-click time, the longest gap inside a run of clicks.
    MultiClick,
    /// 100 ms: a move older than this at the release says the pointer had stopped, so a drag
    /// or a swipe carries no release velocity.
    ReleaseWindow,
    /// 5000 ms: a toast, a banner or the "Sent" pill stays up.
    ToastHold,
    /// 260 ms: the list re-renders after a read toggle (proposed).
    ReadReflow,
    /// 5 s: undo-send's countdown (proposed).
    SendCountdown,
    /// 1 s: one tick of that countdown (proposed).
    SendTick,
    /// 700 ms: the draft autosave debounce (proposed).
    AutosaveDebounce,
    /// 60 ms: focus the To field after the composer page mounts (proposed).
    FocusAfterMount,
    /// 120 ms: a horizontal scroll over a notification has ended once no delta has come for this
    /// long, and the swipe decides (proposed). Blitz forwards no scroll phase, so the
    /// end of a touchpad gesture is a quiet spell, not an event.
    SwipeQuiet,
    /// 1000 ms: type-to-select forgets the letters typed once this long has passed.
    TypeaheadReset,
    /// 300 ms: an `EditSurface` checks the paragraphs that changed once the typing has paused
    /// this long (proposed, design/04-COMPONENTS.md section 50).
    SpellDebounce,
    /// 400 ms: a pending loop shows only if its operation is still running after this, so a
    /// fast join shows no loop at all (design/26-DETAILS.md section 3.4, R4, proposed). It
    /// measures the operation, not motion: Reduced keeps it.
    PendingGrace,
    /// 10 s: after this a pending loop holds its still frame while the operation continues, so
    /// a stuck operation costs 0 frames (design/26 section 3.4, R4, proposed). The only length
    /// a `PendingToken`'s deadline may reach.
    PendingCap,
    /// 900 ms: how long a success check stays drawn before the element rests or leaves
    /// (design/26 section 3.4, R14, proposed). Reading time: Reduced keeps it.
    SettleHold,
}

impl DelayToken {
    /// How long it lasts.
    pub fn delay(self) -> Duration {
        Duration::from_millis(match self {
            DelayToken::TipOpen => 1000,
            DelayToken::CardOpen => 500,
            DelayToken::CardClose => 150,
            DelayToken::LabelOpen => 100,
            DelayToken::HoverWarm => 400,
            DelayToken::LongPress => 500,
            DelayToken::SubmenuOpen => 200,
            DelayToken::TriangleTimeout => 300,
            DelayToken::MultiClick => 500,
            DelayToken::ReleaseWindow => 100,
            DelayToken::ToastHold => 5000,
            DelayToken::ReadReflow => 260,
            DelayToken::SendCountdown => 5000,
            DelayToken::SendTick => 1000,
            DelayToken::AutosaveDebounce => 700,
            DelayToken::FocusAfterMount => 60,
            DelayToken::SwipeQuiet => 120,
            DelayToken::TypeaheadReset => 1000,
            DelayToken::SpellDebounce => 300,
            DelayToken::PendingGrace => 400,
            DelayToken::PendingCap => 10000,
            DelayToken::SettleHold => 900,
        })
    }
}
