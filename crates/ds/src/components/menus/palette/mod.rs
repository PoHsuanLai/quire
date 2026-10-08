//! The command palette: the same menu, bigger and centred, with its groups, stops, reveal and
//! motion, and `machine`, the pure query-and-selection machine a consumer runs beside it.

pub(crate) mod command_palette;
pub mod machine;
pub(crate) mod palette_body;
pub mod palette_claim;
pub(crate) mod palette_expand;
pub mod palette_group;
pub(crate) mod palette_host;
pub(crate) mod palette_lines;
pub mod palette_motion;
pub(crate) mod palette_reveal;
pub(crate) mod palette_rows;
pub(crate) mod palette_select;
pub(crate) mod palette_shown;
pub(crate) mod palette_stops;
