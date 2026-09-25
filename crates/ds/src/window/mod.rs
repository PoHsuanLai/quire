//! A client-decorated window: the host seam its frame drives (`host`), the seam's vocabulary
//! (`vocab`), the frame's timings (`timing`) and the pure machines behind its gestures (`grab`,
//! `hold`). The frame itself is `components::window_frame` (FINDINGS "Window frame").

pub(crate) mod grab;
pub(crate) mod hold;
pub mod host;
pub mod timing;
pub mod vocab;

pub use host::{
    HostWindow, WindowHost, use_window_host, use_window_host_provider, use_window_state,
};
pub use timing::FrameTiming;
pub use vocab::{
    Activation, Fullscreen, Maximized, ResizeEdge, Support, TileError, WindowState, WindowTile,
    Zoom,
};
