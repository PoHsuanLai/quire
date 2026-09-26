//! Keyboard focus: moving it into an element without colliding with the renderer (sill
//! FINDINGS Q43), a caller's handle for giving a field the keyboard again (Q44), selecting a
//! field's text as the focus lands (mailo Phase B, G6), and reaching a field by handle or any
//! element by selector (G8), where the keyboard goes after a click on nothing focusable, and
//! where it goes when a surface that took it leaves (mailo gaps 7).

pub mod caret;
pub mod click;
pub mod field;
pub mod hand_back;
pub mod host;
pub mod request;
pub mod select;
pub mod selector;
pub(crate) mod targets;

pub use caret::{Caret, Collapsed, HostCaret, caret_at};
pub use click::{Fallback, HostClickFocus, HostPressFocus};
pub use field::{FieldHandle, use_field_handle};
pub use hand_back::HostHandBack;
pub use host::{Focused, HostBlur, HostFocus, focus_soon, focus_soon_selecting};
pub use request::{FocusRequest, FocusTicket, use_focus_request};
pub use select::{HostSelect, Select};
pub use selector::{FocusError, Found, HostFind, focus_by_selector};
