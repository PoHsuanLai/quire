//! The design system's appearance: the token table and its `Token` trait, appearance choices and
//! their resolution, materials, Space palettes, fonts, icons, the stylesheet's own sections and
//! the `Kit` seam other crates add theirs through, and the tasks a scope owns (`task`, `busy`),
//! over `ds-core` and nothing else of ours.

// `#[derive(Token)]` names `::ds_style::tokens`, so inside this crate `ds_style` is the crate
// itself.
extern crate self as ds_style;

pub mod appearance;
pub mod busy;
pub mod css;
pub mod emit;
pub mod fonts;
pub mod icon;
pub mod kit;
pub mod look;
pub mod material;
pub mod scale;
pub mod scope;
pub mod space;
pub mod task;
pub mod tokens;
