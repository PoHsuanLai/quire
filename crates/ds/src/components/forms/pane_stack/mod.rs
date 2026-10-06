//! Pane stack: a settings pane that drills into the detail of one of its rows in place
//! (design/34-MODERN-LOOK.md sections 3.5 and 7; System Settings > Internet Accounts > an
//! account). The caller owns the path; the stack draws the page on top of it under a
//! [`PageHeader`](header::PageHeader) and plays the push or the pop.

pub mod header;
pub mod keys;
pub(crate) mod landing;
pub mod path;
pub mod stack;
pub(crate) mod track;
