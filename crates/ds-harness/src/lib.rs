//! The test driver for quire documents on Blitz: a headless document a test builds, sends
//! pointer, key and wheel input to, advances in time, reads and paints, and the one-shot
//! `snapshot` PNGs. Dev-only: nothing that ships an app depends on it.
//!
//! [`Harness`] builds its document with the providers `ds_blitz::launch` installs, so a test runs
//! the code production runs. Its clock is [`Clock::Wall`] or [`Clock::Virtual`]
//! ([`HarnessConfig::with_clock`]). Feature `pdf` adds [`pdf_app`] and [`Harness::pdf`].

mod driver;
mod error;
mod fake_window;
mod frame_view;
mod gpu_device;
mod gpu_diagnostics;
mod gpu_paint;
pub mod harness;
mod harness_backend;
mod harness_clock;
mod harness_config;
mod harness_drop;
mod harness_edit;
mod harness_found;
mod harness_frames;
mod harness_hit;
mod harness_input;
#[cfg(feature = "pdf")]
mod harness_pdf;
mod harness_pointer;
mod harness_settle;
mod harness_style;
mod harness_wheel;
mod harness_window;
mod headless;
mod headless_step;
mod input;
pub mod inset;
mod painter;
mod recorded_window;
mod round_budget;
#[cfg(test)]
mod snap_tests;
pub mod snapshot;

pub use driver::{ClassPresence, DocQuery, Driver, FocusState, Query};
pub use error::HarnessError;
pub use fake_window::{SizerAck, WindowScreen};
pub use frame_view::FrameView;
pub use gpu_diagnostics::GpuDiagnostics;
pub use harness::Harness;
pub use harness_backend::Backend;
pub use harness_clock::Clock;
pub use harness_config::HarnessConfig;
pub use harness_input::HeldButtons;
#[cfg(feature = "pdf")]
pub use harness_pdf::pdf_app;
pub use harness_style::{Part, Srgba};
pub use headless::{Backdrop, Layout};
pub use headless_step::{PhaseOrder, Stepped};
pub use input::{
    ImeInput, Input, KeyInput, PasteChord, PointerAction, PointerInput, RawKeyInput, RawKeyPhase,
};
pub use painter::PaintTime;
pub use recorded_window::WindowHosting;
pub use snapshot::{Viewport, snapshot, snapshot_at, snapshot_placed, snapshot_with};
