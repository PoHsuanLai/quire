//! The style layer's stylesheet sections, generated from the token table: the reset, tokens
//! (`.ds`, `.ds[data-theme=dark]`), accents, materials, shapes, the frame ground and the
//! utilities, and how the whole sheet is laid out as text. The keyframes are motion's and the
//! order is the assembly's. Fonts are not in it: they are registered with the renderer
//! (`crate::fonts`).

pub mod accents_css;
pub mod document;
pub(crate) mod grain;
pub(crate) mod ground_css;
pub(crate) mod materials_css;
pub(crate) mod shape_css;

/// `html, body` transparent; `.ds` carries paper, ink and the UI font.
pub const RESET: &str = include_str!("reset.css");
/// `.ds-truncate` and the other utilities.
pub const UTILITIES: &str = include_str!("utilities.css");
