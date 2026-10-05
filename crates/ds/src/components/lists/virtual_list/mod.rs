//! VirtualList: a list of thousands of rows that mounts only the rows near the viewport.

pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod view;

pub use model::{Change, RowHeight};
pub use view::VirtualList;
