//! The switcher's state, inputs, outputs and timing.

use std::marker::PhantomData;
use std::time::Duration;

use ds_core::machine::Elapsed;
use ds_core::time::stamp::Stamp;

use crate::dir::Dir;

/// The switcher's phase. `sel` indexes the MRU app list the caller provides at every step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Nothing held.
    #[default]
    Hidden,
    /// The chord is down and the panel is not shown yet.
    Armed {
        /// When the panel shows: the chord's time plus the show delay.
        until: Stamp,
        /// The app the release would activate.
        sel: usize,
    },
    /// The panel is up.
    Shown {
        /// The highlighted app.
        sel: usize,
    },
}

/// The switcher machine over the caller's app key `K`: its [`Phase`]. `K` appears only in the
/// outputs and in the app list the caller provides as the step's context (the current app first),
/// so the compositor runs it over its own app ids and the shell over its launcher's.
pub struct Switcher<K> {
    pub(crate) phase: Phase,
    keys: PhantomData<fn() -> K>,
}

impl<K> Switcher<K> {
    /// A switcher in `phase`.
    pub fn in_phase(phase: Phase) -> Self {
        Switcher {
            phase,
            keys: PhantomData,
        }
    }

    /// A switcher with nothing held.
    pub fn hidden() -> Self {
        Switcher::in_phase(Phase::Hidden)
    }

    /// Where it is.
    pub fn phase(&self) -> Phase {
        self.phase
    }
}

impl<K> Clone for Switcher<K> {
    fn clone(&self) -> Self {
        Switcher::in_phase(self.phase)
    }
}

impl<K> PartialEq for Switcher<K> {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase
    }
}

impl<K> Eq for Switcher<K> {}

impl<K> std::fmt::Debug for Switcher<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Switcher").field(&self.phase).finish()
    }
}

impl<K> Default for Switcher<K> {
    fn default() -> Self {
        Switcher::hidden()
    }
}

/// A key pressed while the switcher is up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwKey {
    /// Next app.
    Tab,
    /// Previous app.
    ShiftTab,
    /// The grave key: previous app.
    Grave,
    /// Previous app.
    Left,
    /// Next app.
    Right,
    /// App Exposé for the selected app.
    Up,
    /// App Exposé for the selected app.
    Down,
    /// Quit the selected app; the panel stays.
    Q,
    /// Hide the selected app; the panel stays.
    H,
    /// Cancel: nothing is activated.
    Escape,
}

/// One input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwIn {
    /// The switcher chord (Command-Tab forward, Shift-Command-Tab back).
    Chord(Dir),
    /// The chord's modifier went up.
    ModifierReleased,
    /// A key while the chord is held.
    Key(SwKey),
    /// The pointer is over the app at this index in the panel.
    Hover(usize),
    /// The app at this index in the panel was clicked.
    Click(usize),
    /// The deadline asked for by `wake` came due, or the caller's app list changed and the
    /// machine should look again (no apps hides the panel).
    Elapsed,
}

impl From<Elapsed> for SwIn {
    fn from(_: Elapsed) -> SwIn {
        SwIn::Elapsed
    }
}

/// One thing the caller must do. `K` is the caller's app key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwOut<K> {
    /// Show the panel.
    Show,
    /// Hide the panel.
    Hide,
    /// Highlight the app at this index.
    Select(usize),
    /// Activate the app (unhide it, or ask a windowless app to reopen).
    Activate(K),
    /// Quit the app.
    Quit(K),
    /// Hide the app.
    HideApp(K),
    /// Open App Exposé for the app.
    Expose(K),
}

/// The switcher's timing (`switcher.show_delay_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitcherParams {
    /// How long the chord is held before the panel appears; a release inside it switches with
    /// no panel.
    pub show_delay: Duration,
}
