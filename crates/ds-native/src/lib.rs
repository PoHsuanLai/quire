//! Blitz glue for quire: launching an app, registering the faces, headless snapshots to PNG, and
//! a harness for event-driven tests. The only quire crate that names the blitz crates.

pub mod error;
pub mod fonts;
pub mod harness;
pub mod launch;
pub mod snapshot;

pub use error::NativeError;
pub use fonts::register_fonts;
pub use harness::Harness;
pub use launch::{AppConfig, launch};
pub use snapshot::{Viewport, snapshot};

use anyrender as _;
use anyrender_vello_cpu as _;
use anyrender_vello_hybrid as _;
use blitz_dom as _;
use blitz_paint as _;
use blitz_traits as _;
use dioxus_native_dom as _;
use wgpu as _;
