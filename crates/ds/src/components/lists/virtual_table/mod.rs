//! VirtualTable: a table of any number of rows that never builds them. Only the rows in the
//! viewport (and a few around it) are asked of the caller's cell callback.

pub(crate) mod drag_model;
pub(crate) mod model;
pub(crate) mod outline;
pub(crate) mod row_drag;
pub(crate) mod split;
pub(crate) mod view;

pub use drag_model::{RowDrop, RowDropKind, RowDropTarget, RowGaps};
pub use outline::{OutlineKind, OutlineRow, OutlineToggle, ToggleReach};
pub use split::columns_id;
pub use view::{VirtualTable, row_pitch};
