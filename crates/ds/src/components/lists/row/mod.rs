//! Row: the one line of a list and everything a row is made of (leading element, title, detail,
//! accessory, shape, chord, action).

pub(crate) mod accessory;
pub(crate) mod action;
pub(crate) mod chord;
pub(crate) mod leading;
pub(crate) mod marks;
pub(crate) mod motion;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub(crate) mod row;
pub(crate) mod shape;
pub(crate) mod shape_view;
pub(crate) mod size;
