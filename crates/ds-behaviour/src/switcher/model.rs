//! The switcher's state, inputs, outputs and timing.

use std::time::Duration;

use ds_core::time::stamp::Stamp;

use crate::dir::Dir;

/// The switcher's phase. `sel` indexes the MRU app list passed to every
/// [`step`](super::step).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Switcher {
    /// Nothing held.
    #[default]
    Hidden,
    /// The chord is down and the panel is not shown yet.
    Armed {
        /// When the chord went down; the panel shows at `since` plus the show delay.
        since: Stamp,
        /// The app the release would activate.
        sel: usize,
    },
    /// The panel is up.
    Shown {
        /// The highlighted app.
        sel: usize,
    },
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
    /// The tick the machine asked for with [`SwOut::RequestTick`] came due.
    Tick,
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
    /// Step the machine with [`SwIn::Tick`] at this time.
    RequestTick(Stamp),
}

/// The switcher's timing (`switcher.show_delay_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitcherParams {
    /// How long the chord is held before the panel appears; a release inside it switches with
    /// no panel.
    pub show_delay: Duration,
}
