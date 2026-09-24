//! The stylesheet, generated from the token table. Cascade order: reset, tokens (`.ds`,
//! `.ds[data-theme=dark]`), `.ds[data-motion]`, accents, materials, keyframes with their
//! `X`/`X--b` aliases, utilities, components in a fixed list. Fonts are not in it: they are
//! registered with the renderer (`crate::fonts`).

pub mod accents_css;
pub mod emit;
pub mod ground_css;
pub mod materials_css;
pub mod motion_css;
pub mod stylesheet;
pub mod tokens_css;

pub use stylesheet::stylesheet;

/// `html, body` transparent; `.ds` carries paper, ink and the UI font.
pub const RESET: &str = include_str!("reset.css");
/// The canonical keyframes, verbatim from design/05-MOTION.md section 4 (without `filter`).
pub const MOTION: &str = include_str!("motion.css");
/// `.ds-truncate` and the other utilities.
pub const UTILITIES: &str = include_str!("utilities.css");
/// The frame's grain tile, a 128 px PNG built by `build.rs` from the prototype's generator
/// (design/03-COLOR.md section 8), as a `data:image/png;base64,…` URI. The stylesheet paints it
/// on `.ds-grain`; a renderer needs a `data:` net provider to load it (spike S8).
pub const GRAIN_PNG: &str = include_str!(concat!(env!("OUT_DIR"), "/grain.uri"));
