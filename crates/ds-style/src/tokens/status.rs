//! The bar's status item geometry: the two custom properties `MenuBarItem` reads. They are
//! stylesheet tokens (22 px box, 16 px glyph by default); a look or the person's own sheet moves
//! them, and Rust that needs the box reads it with `ds_blitz::Resolved`.

use crate::tokens::name::VarName;

/// The property the item's box (its height) is read from.
pub const BOX_VAR: VarName = VarName("--bar-status-box");
/// The property the glyph inside it is read from.
pub const GLYPH_VAR: VarName = VarName("--bar-status-glyph");
