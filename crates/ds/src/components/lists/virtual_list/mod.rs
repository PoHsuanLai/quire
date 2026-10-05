//! VirtualList: a list of thousands of rows that mounts only the rows near the viewport.

pub(crate) mod model;
pub(crate) mod view;

pub use model::RowHeight;
pub use view::VirtualList;
