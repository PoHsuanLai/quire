//! What passes between the window's IME and a surface: its events, its switch, and the
//! registration to cancel.

/// What the IME tells the focused surface, as the host receives it (winit's `Ime`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// The IME attached to the window.
    Enabled,
    /// The preedit is now `text` (empty: cleared), with the IME's cursor as UTF-8 byte offsets.
    Preedit {
        /// The text being composed.
        text: String,
        /// The IME's cursor or highlight in it.
        cursor: Option<(usize, usize)>,
    },
    /// Insert `text`.
    Commit(String),
    /// The IME detached.
    Disabled,
}

/// Whether the window's IME is on for the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImeSwitch {
    /// On: the surface has the keyboard.
    On,
    /// Off: it lost it.
    Off,
}

/// A surface's registration for IME events, to cancel as it unmounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImeListener(pub u64);
