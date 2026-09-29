//! Text that has to fit: Blitz has no `text-overflow: ellipsis` or `line-clamp`.

pub mod clip;

pub use clip::clip_chars;
