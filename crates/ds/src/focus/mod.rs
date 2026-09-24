//! Keyboard focus: moving it into an element without colliding with the renderer (sill
//! FINDINGS Q43), a caller's handle for giving a field the keyboard again (Q44), and selecting a
//! field's text as the focus lands (mailo Phase B, G6).

pub mod host;
pub mod request;
pub mod select;

pub use host::{Focused, HostFocus, focus_soon, focus_soon_selecting};
pub use request::{FocusRequest, FocusTicket, use_focus_request};
pub use select::{HostSelect, Select};
