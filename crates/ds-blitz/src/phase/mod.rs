//! The frame phase: one step per frame, run by the window loop after layout with the document
//! unborrowed, that applies the writes components queued and publishes what the document now
//! shows into the signals components watch, only when it changed. A component never reads or
//! writes the Blitz document itself, so nothing waits for a renderer that holds it.
//!
//! A caret's box is also published before a frame paints, when the caret moved (`Phase::early`),
//! so it is drawn in the same frame as the text it follows.
//!
//! The window loop runs it right after a frame is drawn (layout is fresh then), and again after
//! the document was updated by a wake-up, when layout is not: a write applied then is kept and
//! applied again after the next layout, so it clamps against the content as it is by then.

mod book;
mod caret;
mod read;
mod run;

pub use caret::Early;
pub(crate) use read::border_box;
pub use run::{Layout, Phase, Ran};
