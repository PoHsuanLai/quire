//! Blitz glue for quire: launching an app, registering the faces, headless snapshots to PNG, and
//! a harness for event-driven tests. The only quire crate that names the blitz crates.

mod data_url;
pub mod error;
pub mod fonts;
pub mod harness;
mod headless;
mod host;
pub mod launch;
mod measure;
mod net;
mod scheme;
pub mod snapshot;
mod wake;

pub use error::NativeError;
pub use fonts::{font_context, register_fonts};
pub use harness::Harness;
pub use launch::{AppConfig, launch};
pub use snapshot::{Viewport, snapshot, snapshot_at};

// The window renderer dioxus-native runs on; named here so the pinned versions stay the ones
// the render stack resolves.
use anyrender_vello_hybrid as _;
use wgpu as _;
