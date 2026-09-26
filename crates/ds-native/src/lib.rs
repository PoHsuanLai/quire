//! Blitz glue for quire: launching an app, registering the faces, headless snapshots to PNG,
//! PDF output (`pdf`, `pdf_app`, and `print_dialog` behind the `print` feature), a harness for
//! event-driven tests, and the device-pixel layout snap (`snap`). The only quire crate that
//! names the blitz crates.
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
mod drop_hit;
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
mod focus_chain;
mod focus_keep;
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
mod harness_drop;
mod harness_edit;
mod harness_hit;
mod harness_input;
mod harness_settle;
mod harness_wheel;
mod headless;
mod host;
mod hover_replay;
mod hover_sync;
mod install;
pub mod launch;
pub mod measure;
mod memory_shell;
mod native_providers;
mod net;
mod net_policy;
mod node_ref;
mod open_window;
mod origin;
mod pdf;
#[cfg(feature = "pdf-thumb")]
mod pdf_thumb;
#[cfg(feature = "print")]
mod print;
mod route;
mod runtime;
mod scheme;
mod setup;
pub mod snap;
#[cfg(test)]
mod snap_tests;
pub mod snapshot;
mod wake;
pub mod window;
mod window_build;
mod window_drop;
mod window_hover;
mod window_place;
mod window_requests;
mod window_shell;

pub use app_id::AppId;
pub use click_focus::{CLICK_FOCUS, FocusFallback, PRESS_FOCUS};
pub use contexts::RootContexts;
pub use error::{NativeError, OpenWindowError};
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
pub use open_window::{WindowHandle, WindowSpec, open_window, open_window_with};
pub use origin::{FrameId, RequestOrigin};
pub use pdf::{Margins, PageSize, PageSpec, PdfError, Pt, pdf, pdf_app};
#[cfg(feature = "pdf-thumb")]
pub use pdf_thumb::{
    DeviceBox, PdfFileThumb, QUEUE_DEPTH, THUMB_CACHE_ENTRIES, ThumbKey, ThumbRequest,
    pdf_thumb_blocking, pdf_thumb_bytes, pdf_thumb_cached, pdf_thumb_rasters, use_pdf_page,
};
#[cfg(feature = "print")]
pub use print::{PrintError, PrintOutcome, print_dialog};
pub use snap::snap_to_device;
pub use snapshot::{Viewport, snapshot, snapshot_at, snapshot_with};
pub use window::{Decorations, WinitWindow};
pub use window_requests::WindowLife;

// The window renderer dioxus-native runs on; named here so the pinned versions stay the ones
// the render stack resolves.
use anyrender_vello_hybrid as _;
use wgpu as _;
