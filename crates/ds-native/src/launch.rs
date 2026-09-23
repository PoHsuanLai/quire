//! Running a quire app on Blitz: the document, the font registration, the input modality
//! (`ds::HostModality`), a `data:` net provider for mask and background images (spike S7/S8),
//! and a redraw when a timer or an image lands.
//!
//! The window is blitz's portable `dioxus-native` shell (winit), so an app runs the same on any
//! OS; `crate::host` wraps the app to supply what quire needs on top of it.

use crate::fonts::font_context;
use crate::host::{Host, HostProps};
use dioxus::prelude::*;
use dioxus_native::{LogicalSize, WindowAttributes};

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
    // Held until this call returns, which does not happen until the window closes — i.e., for
    // the process's life. See `crate::runtime` for why a host thread must enter Tokio at all.
    let _runtime = crate::runtime::enter();
    let window = WindowAttributes::default()
        .with_title(config.title)
        .with_surface_size(LogicalSize::new(config.width, config.height));
    let native = dioxus_native::Config::new()
        .with_window_attributes(window)
        .with_font_ctx(font_context());
    dioxus_native::launch_cfg_with_props(
        Host,
        HostProps::new(app),
        Vec::new(),
        vec![Box::new(native)],
    );
}
