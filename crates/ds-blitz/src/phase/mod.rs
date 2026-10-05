//! The frame phase: one step per frame, run by the window loop after layout with the document
//! unborrowed, that applies the writes components queued and publishes what the document now
//! shows into the signals components watch, only when it changed. A component never reads or
//! writes the Blitz document itself, so nothing waits for a renderer that holds it.
//!
//! The window loop runs it right after a frame is drawn (layout is fresh then), and again after
//! the document was updated by a wake-up, when layout is not: a write applied then is kept and
//! applied again after the next layout, so it clamps against the content as it is by then.

mod book;
mod read;
mod run;

pub(crate) use read::border_box;
pub use run::{Layout, Phase, Ran};
