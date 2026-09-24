//! Keyboard focus: moving it into an element without colliding with the renderer (sill
//! FINDINGS Q43), and a caller's handle for giving a field the keyboard again (Q44).

pub mod host;
pub mod request;

pub use host::{Focused, HostFocus};
pub use request::{FocusRequest, FocusTicket, use_focus_request};
