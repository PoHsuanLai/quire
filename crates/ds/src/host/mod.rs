//! What a component asks of the document it is drawn in: measuring an element's rect and scrolling
//! one into view, and the vocabulary the host seams speak, through the seams a host provides.

pub mod captured;
pub mod caret;
pub mod document;
pub mod drop_hit;
pub mod fallback;
pub mod focused;
pub mod found;
pub mod gesture;
pub mod hand_back;
pub mod ime;
pub mod layout;
pub mod measure;
pub mod no_host;
pub mod parts;
pub mod pasted;
pub mod phase;
pub mod pointer_capture;
pub mod position;
pub mod probe;
pub mod resized;
pub mod reveal;
pub mod signals;
