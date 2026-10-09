//! A client-decorated window: the host seam its frame drives (`host`), the seam's vocabulary
//! (`vocab`), the window's icon (`icon`), the frame's timings (`timing`) and the pure machines behind its gestures (`grab`,
//! `hold`). The frame itself is `components::window_frame` (FINDINGS "Window frame").

pub(crate) mod grab;
pub(crate) mod hold;
pub mod host;
pub mod icon;
pub mod tiled;
pub(crate) mod timing;
pub mod vocab;
