//! Running a quire app on Blitz: the document, the font registration, the input modality
//! (`ds::HostModality`), a `data:` net provider for mask and background images (spike S7/S8),
//! and a redraw when a timer or an image lands.
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use dioxus::prelude::*;

/// How the window starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// The window title.
    pub title: String,
    /// The initial width in logical pixels.
    pub width: u32,
    /// The initial height in logical pixels.
    pub height: u32,
}

/// Run `app` until its window closes.
pub fn launch(app: fn() -> Element, config: AppConfig) {
    todo!()
}
