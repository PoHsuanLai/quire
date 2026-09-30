//! Blitz glue for quire: launching an app and opening its windows, registering the faces, the
//! clipboard and the host seams (`provide_host`), PDF output (feature `pdf`), printing (feature
//! `print`), spellchecking (feature `spell`), and the device-pixel layout snap (`snap`). The only
//! quire crate that names the blitz crates; the test driver lives in `ds-harness`, which builds
//! its document from the parts in [`seam`].
//!
//! It is also the only quire crate that may depend on `tokio` (`scripts/check-boundary.sh`
//! forbids it to `ds`): `launch` owns the process-wide runtime (`launch::runtime`), so the
//! `TokioSpawner` it hands `ds_settings::use_environment` has a runtime to run its portal and
//! file-watch tasks on, and `enter_runtime` lends it to a test driver.

mod app_id;
mod blitz_host;
mod click_focus;
pub mod clipboard;
mod contexts;
mod drop_hit;
mod edit;
mod edit_align;
mod edit_geometry;
mod edit_hit;
mod edit_ime;
mod edit_locate;
mod edit_tree;
mod edit_window;
pub mod error;
mod focus;
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
pub mod frames;
mod host;
mod hover_replay;
mod hover_sync;
mod install;
pub mod launch;
mod measure;
mod memory_shell;
mod native_providers;
mod net;
mod net_policy;
mod node_ref;
mod open_window;
mod origin;
#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "pdf")]
mod pdf_thumb;
#[cfg(feature = "print")]
mod print;
mod reveal;
mod route;
mod scheme;
pub mod seam;
mod setup;
pub mod snap;
#[cfg(feature = "spell")]
pub mod spell;
mod wake;
pub mod window;
mod window_build;
mod window_drop;
mod window_hover;
mod window_place;
mod window_requests;
mod window_shell;

pub use app_id::AppId;
pub use blitz_host::provide_host;
pub use blitz_kit::adapter::{ADAPTER_ENV, AdapterPref};
pub use click_focus::FocusFallback;
pub use contexts::RootContexts;
pub use error::{NativeError, OpenWindowError};
pub use fonts::font_context;
pub use frame_hover::{FrameHover, FrameHoverHandler, FrameLinkHover, HoverPhase};
pub use frame_links::{FrameLink, FrameLinkHandler, FrameLinks};
pub use frame_tag::FrameTag;
pub use launch::{AppConfig, RuntimeGuard, TokioSpawner, enter_runtime, launch};
pub use net_policy::{AppNet, NetDecision, NetPolicy, NetReply, NetRequest};
pub use open_window::{WindowHandle, WindowSpec, open_window, open_window_with};
pub use origin::{FrameId, RequestOrigin};
#[cfg(feature = "pdf")]
pub use pdf::{
    ContentPx, Margins, PageSize, PageSpec, PdfError, Pt, content_size, pdf, print_document,
    print_viewport,
};
#[cfg(feature = "pdf")]
pub use pdf_thumb::{
    DeviceBox, PdfFileThumb, QUEUE_DEPTH, THUMB_CACHE_ENTRIES, ThumbKey, ThumbRequest,
    pdf_thumb_blocking, pdf_thumb_bytes, pdf_thumb_cached, pdf_thumb_rasters, use_pdf_page,
};
#[cfg(feature = "print")]
pub use print::{PrintError, PrintOutcome, print_dialog};
pub use snap::snap_to_device;
pub use window::{Decorations, WinitWindow};
pub use window_requests::WindowLife;

// The window renderer dioxus-native runs on; named here so the pinned versions stay the ones
// the render stack resolves.
use anyrender_vello_hybrid as _;
use wgpu as _;
