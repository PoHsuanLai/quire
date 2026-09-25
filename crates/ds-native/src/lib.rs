//! Blitz glue for quire: launching an app, registering the faces, headless snapshots to PNG, a
//! harness for event-driven tests, and the device-pixel layout snap (`snap`). The only quire
//! crate that names the blitz crates.
//!
//! It is also the only quire crate that may depend on `tokio` (`scripts/check-boundary.sh`
//! forbids it to `ds`): `launch` and `Harness` each enter a process-wide runtime (`crate::
//! runtime`) so `ds_settings::use_environment` can spawn its portal and file-watch tasks without
//! panicking.

mod app_id;
mod click_focus;
pub mod clipboard;
mod contexts;
mod data_url;
pub mod edit;
mod edit_align;
mod edit_geometry;
mod edit_hit;
mod edit_ime;
mod edit_locate;
mod edit_tree;
mod edit_window;
pub mod error;
pub mod focus;
pub mod fonts;
mod frame_anchor;
mod frame_book;
mod frame_hit;
mod frame_hover;
mod frame_links;
mod frame_tag;
mod frame_tree;
mod frame_view;
pub mod frames;
pub mod harness;
mod harness_config;
mod harness_edit;
mod harness_hit;
mod harness_input;
mod headless;
mod host;
mod install;
pub mod launch;
pub mod measure;
mod memory_shell;
mod net;
mod net_policy;
mod node_ref;
mod origin;
mod route;
mod runtime;
mod scheme;
mod setup;
pub mod snap;
#[cfg(test)]
mod snap_tests;
pub mod snapshot;
mod wake;
mod window_hover;

pub use app_id::AppId;
pub use click_focus::{CLICK_FOCUS, FocusFallback};
pub use contexts::RootContexts;
pub use error::NativeError;
pub use fonts::{font_context, register_fonts};
pub use frame_hover::{FrameHover, FrameHoverHandler, FrameLinkHover, HoverPhase};
pub use frame_links::{FrameLink, FrameLinkHandler, FrameLinks};
pub use frame_tag::FrameTag;
pub use frame_view::FrameView;
pub use harness::Harness;
pub use harness_config::HarnessConfig;
pub use harness_input::HeldButtons;
pub use headless::Backdrop;
pub use launch::{AppConfig, launch};
pub use net_policy::{AppNet, NetDecision, NetPolicy, NetReply, NetRequest};
pub use origin::{FrameId, RequestOrigin};
pub use snap::snap_to_device;
pub use snapshot::{Viewport, snapshot, snapshot_at, snapshot_with};

// The window renderer dioxus-native runs on; named here so the pinned versions stay the ones
// the render stack resolves.
use anyrender_vello_hybrid as _;
use wgpu as _;
