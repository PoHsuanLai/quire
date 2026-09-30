//! The halo ring: a pending loop drawn around its positioned parent (design/04-COMPONENTS.md
//! section 15), for a part that reads the frame itself (the preview pane, which also swaps its
//! media for the pending look while the loop shows, and a settings row's working ring). The
//! spinner of a control is [`ProgressIndicator`](crate::components::controls::progress::view::ProgressIndicator).
//! `data-pending` is `idle` (not shown) or `step` (moving: `--turn` is the ring's angle).

use dioxus::prelude::*;
use ds_motion::detail::pending::PendingFrame;

pub(crate) use crate::components::controls::progress::spokes::SPIN;

/// The ring at `frame`.
pub(crate) fn ring(frame: PendingFrame) -> Element {
    let turn = match frame {
        PendingFrame::Step(n) => Some(format!("--turn:{}deg", u32::from(n) * 30)),
        PendingFrame::Idle => None,
    };
    rsx! {
        span {
            class: "ds-spinner",
            "data-pending": frame.slug(),
            style: turn,
            "aria-hidden": "true",
        }
    }
}
