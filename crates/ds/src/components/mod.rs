//! The general components, by concept, one `<name>.rs` and `<name>.css` pair each
//! (design/04-COMPONENTS.md); mail's own are in `app`, the shell's in the shell layer.

pub(crate) mod app;
pub mod chrome;
pub mod content;
pub mod controls;
pub(crate) mod editor;
pub mod fields;
pub mod lists;
pub(crate) mod menus;
pub mod overlays;
