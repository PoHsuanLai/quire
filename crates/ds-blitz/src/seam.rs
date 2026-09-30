//! The parts a headless Blitz host wires a document from: `ds-harness` builds its document out of
//! exactly what a window's does (the providers, the frame bookkeeping, the hover repair), so a
//! test runs the code production runs. An app has no use for these; it uses `launch`.

pub use crate::blitz_host::{Provided, Wiring};
pub use crate::edit_hit::hit as edit_hit;
pub use crate::edit_ime::EditListeners;
pub use crate::focus::finder as focus_finder;
pub use crate::focus_keep::{FocusKeeper, Kept, keep};
pub use crate::frame_anchor::LinkFacts;
pub use crate::frame_book::FrameBook;
pub use crate::frame_hit::{LinkUnder, link_under};
pub use crate::frame_hover::{HoverTracker, report as report_frame_hover};
pub use crate::frame_links::{LinkInbox, frame_links, read_link};
pub use crate::frame_tree::live_frames;
pub use crate::frames::FrameParser;
pub use crate::memory_shell::MemoryShell;
pub use crate::net::DsNet;
pub use crate::node_ref::DocRef;
pub use crate::scheme::follow_root as follow_scheme;
pub use crate::setup::Setup;
pub use crate::wake::Wakeup;
