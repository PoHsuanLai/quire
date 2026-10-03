//! Menus: the one menu and its items, the pop-up button that opens one, the command palette
//! and pick list built on the same rows, and the conformance file an app's menu items and
//! shortcuts are written to.

pub mod item;
pub(crate) mod menu;
pub mod menu_bar;
pub(crate) mod menu_match;
pub mod palette;
pub mod pick_list;
pub mod pop_up_button;
pub mod ui_manifest;
