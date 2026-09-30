//! The Rust-only timer lengths (design/30-CATALOGUE.md section 1.2): intent and reading time,
//! never scaled by the motion level.

use ds_core::word::Word;
use std::time::Duration;

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
    /// 70 ms: one half of a menu item's blink: it is lit for this long, then unlit for this
    /// long, twice, before the menu closes on the pick.
    MenuBlink,
    /// 500 ms: the system double-click time, the longest gap inside a run of clicks.
    MultiClick,
    /// 100 ms: a move older than this at the release says the pointer had stopped, so a drag
    /// or a swipe carries no release velocity.
    ReleaseWindow,
    /// 5000 ms: a toast, a banner or the "Sent" pill stays up.
    ToastHold,
    /// 260 ms: the list re-renders after a read toggle (proposed).
    ReadReflow,
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
            DelayToken::MenuBlink => 70,
            DelayToken::MultiClick => 500,
            DelayToken::ReleaseWindow => 100,
            DelayToken::ToastHold => 5000,
            DelayToken::ReadReflow => 260,
            DelayToken::AutosaveDebounce => 700,
            DelayToken::FocusAfterMount => 60,
            DelayToken::SwipeQuiet => 120,
            DelayToken::TypeaheadReset => 1000,
            DelayToken::SpellDebounce => 300,
        })
    }
}
