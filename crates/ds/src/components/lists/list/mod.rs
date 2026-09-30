//! List: the container rows live in, its items and its keys.

pub(crate) mod entry;
pub(crate) mod keys;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub(crate) mod list;
pub mod model;
