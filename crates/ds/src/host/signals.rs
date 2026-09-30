//! What the host tells the tree as it changes: how the person last drove it, the device scale of
//! the output it is on, and whether its window is the one they are working in.

use dioxus::prelude::*;
use ds_core::geometry::scale::Scale;
use ds_core::vocab::{Activity, InputModality};

/// The host's live values, provided as root context by whatever sees the raw input and the window
/// (`launch`, the harness, a shell's surface root). `Ds` stamps them on `.ds` and sizes the pixel
/// tokens by the scale. Signals, because each changes while the tree lives: a key press, a window
/// moved to another output, focus lost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostSignals {
    /// How the person last drove the window.
    pub modality: Signal<InputModality>,
    /// The device scale of the output the window is on.
    pub scale: Signal<Scale>,
    /// Whether the window is the one the person is working in.
    pub activity: Signal<Activity>,
}
