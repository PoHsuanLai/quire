//! Keyboard focus: moving it into an element without colliding with the renderer,
//! a caller's handle for giving a field the keyboard again, selecting a
//! field's text as the focus lands, and reaching a field by handle or any
//! element by selector, where the keyboard goes after a click on nothing focusable, and
//! where it goes when a surface that took it leaves.

pub mod click;
pub mod field;
pub mod request;
pub mod select;
pub mod selector;
pub mod soon;
pub(crate) mod targets;
