//! VirtualTable: a table of any number of rows that never builds them. Only the rows in the
//! viewport (and a few around it) are asked of the caller's cell callback.

pub(crate) mod model;
pub(crate) mod outline;
pub(crate) mod split;
pub(crate) mod view;

pub use outline::{OutlineKind, OutlineRow, OutlineToggle, ToggleReach};
pub use split::columns_id;
pub use view::{VirtualTable, row_pitch};
