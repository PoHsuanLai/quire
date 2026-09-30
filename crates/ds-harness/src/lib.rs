//! The test driver for quire documents on Blitz: a headless document a test builds, sends
//! pointer, key and wheel input to, advances in time, reads and paints, and the one-shot
//! `snapshot` PNGs. Dev-only: nothing that ships an app depends on it.
//!
//! [`Harness`] builds its document with the providers `ds_blitz::launch` installs, so a test runs
//! the code production runs. Its clock is [`Clock::Wall`] or [`Clock::Virtual`]
//! ([`HarnessConfig::with_clock`]). Feature `pdf` adds [`pdf_app`] and [`Harness::pdf`].

mod frame_view;
mod gpu_paint;
pub mod harness;
mod harness_backend;
mod harness_clock;
mod harness_config;
mod harness_drop;
mod harness_edit;
mod harness_hit;
mod harness_input;
#[cfg(feature = "pdf")]
mod harness_pdf;
mod harness_settle;
mod harness_style;
mod harness_wheel;
mod headless;
mod painter;
#[cfg(test)]
mod snap_tests;
pub mod snapshot;

pub use frame_view::FrameView;
pub use harness::Harness;
pub use harness_backend::Backend;
pub use harness_clock::Clock;
pub use harness_config::HarnessConfig;
pub use harness_input::HeldButtons;
#[cfg(feature = "pdf")]
pub use harness_pdf::pdf_app;
pub use harness_style::{Part, Srgba};
pub use headless::Backdrop;
pub use painter::PaintTime;
pub use snapshot::{Viewport, snapshot, snapshot_at, snapshot_placed, snapshot_with};
