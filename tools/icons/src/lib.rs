//! App-icon post-processing for the bake-off and, later, the set (design/08-ICONS.md 2, 3.7).
//!
//! Every function here is pure: images in, images out. Only `main.rs` reads and writes files.

mod bevel;
mod board;
mod color;
mod compose;
mod dialect;
mod emblem;
mod error;
mod export;
mod family;
mod fit;
mod font;
mod grain;
mod key;
mod mark;
mod plane;
mod plate;
mod retint;
mod sheet;
mod ship;
mod spec;
mod swatch;
mod template;

pub use bevel::{Bevel, finish, plate_face};
pub use board::{face_cell, look_cell, render_icon, space_tint};
pub use color::{Oklab, Srgb8, delta_e, in_srgb, linear_to_srgb, oklab, srgb, srgb_to_linear};
pub use compose::{Shadow, compose, compose_on, drop_shadow, over};
pub use dialect::{CHROMA_CAP, ChromaCap, Dialect, Lch, PALETTE, Roles, Shift, Tint, roles};
pub use emblem::{EMBOSS_FROM, Look, emblem, emblem_object};
pub use error::IconsError;
pub use export::{EXPORT_SIZES, Export, export, grid_for};
pub use family::{Family, Stops};
pub use fit::{Rect, alpha_bbox, fit_object};
pub use font::{draw_text, text_width};
pub use grain::{GRAIN_TILE, apply_grain, grain_tile};
pub use key::{Key, key_background};
pub use mark::{BAND, Corner, Corners, Heading, Pt, Shape, distance};
pub use plane::Plane;
pub use plate::{PlateGrid, gradient, plate_mask};
pub use retint::retint;
pub use sheet::{Cell, Sheet, SheetStyle, build_sheet, size_strip, strip, strip_sheet};
pub use ship::{
    FACE_CROP, Manifest, Prepared, SHIP_PX, ShipApp, Source, Style, face_icon, klein_face,
    ship_icon, style_look,
};
pub use spec::{Grain, Layer, Relief, Role, Spec, parse_spec};
pub use swatch::palette_board;
pub use template::{Fraction, Template};
