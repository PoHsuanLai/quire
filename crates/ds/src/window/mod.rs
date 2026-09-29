//! A client-decorated window: the host seam its frame drives (`host`), the seam's vocabulary
//! (`vocab`), the frame's timings (`timing`) and the pure machines behind its gestures (`grab`,
//! `hold`). The frame itself is `components::window_frame` (FINDINGS "Window frame").

pub(crate) mod grab;
pub(crate) mod hold;
pub(crate) mod host;
pub(crate) mod timing;
pub(crate) mod vocab;
