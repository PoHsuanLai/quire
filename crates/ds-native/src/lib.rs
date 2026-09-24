//! Blitz glue for quire: launching an app, registering the faces, headless snapshots to PNG, a
//! harness for event-driven tests, and the device-pixel layout snap (`snap`). The only quire
//! crate that names the blitz crates.
//!
//! It is also the only quire crate that may depend on `tokio` (`scripts/check-boundary.sh`
//! forbids it to `ds`): `launch` and `Harness` each enter a process-wide runtime (`crate::
//! runtime`) so `ds_settings::use_environment` can spawn its portal and file-watch tasks without
//! panicking.

mod data_url;
pub mod error;
pub mod focus;
pub mod fonts;
pub mod harness;
mod headless;
mod host;
pub mod launch;
pub mod measure;
mod net;
mod runtime;
mod scheme;
pub mod snap;
#[cfg(test)]
mod snap_tests;
pub mod snapshot;
mod wake;

pub use error::NativeError;
pub use fonts::{font_context, register_fonts};
pub use harness::Harness;
pub use headless::Backdrop;
pub use launch::{AppConfig, launch};
pub use snap::snap_to_device;
pub use snapshot::{Viewport, snapshot, snapshot_at};

// The window renderer dioxus-native runs on; named here so the pinned versions stay the ones
// the render stack resolves.
use anyrender_vello_hybrid as _;
use wgpu as _;
