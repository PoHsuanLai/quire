//! Menu: the floating list of commands, its panels, its keys and its pick.

pub(crate) mod active;
pub(crate) mod blink;
pub(crate) mod choices;
pub(crate) mod cursor;
pub(crate) mod decide;
pub(crate) mod hand_back;
pub(crate) mod keys;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub(crate) mod menu;
pub(crate) mod panel;
pub(crate) mod pick;
pub(crate) mod placement;
pub(crate) mod submenu;
pub(crate) mod surface;
pub(crate) mod tracker;
