//! MenuItem: what a menu lists, as data, and how it draws.

pub(crate) mod context;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub(crate) mod item;
pub(crate) mod lines;
pub(crate) mod view;
