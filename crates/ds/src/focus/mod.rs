//! Keyboard focus: moving it into an element without colliding with the renderer,
//! a caller's handle for giving a field the keyboard again, selecting a
//! field's text as the focus lands, and reaching a field by handle or any
//! element by selector, where the keyboard goes after a click on nothing focusable, and
//! where it goes when a surface that took it leaves.

pub mod caret;
pub mod click;
pub mod field;
pub mod hand_back;
pub mod host;
pub mod request;
pub mod select;
pub mod selector;
pub(crate) mod targets;

pub use caret::{
    Caret, Collapsed, FieldSelection, HostCaret, HostPlaceCaret, HostSelection, InitialCaret,
    caret_at,
};
pub use click::{Fallback, HostClickFocus, HostPressFocus};
pub use field::{FieldHandle, use_field_handle};
pub use hand_back::HostHandBack;
pub use host::{Focused, HostBlur, HostFocus, focus_soon, focus_soon_selecting};
pub use request::{FocusRequest, FocusTicket, use_focus_request};
pub use select::{HostSelect, Select};
pub use selector::{FocusError, Found, HostFind, focus_by_selector};
