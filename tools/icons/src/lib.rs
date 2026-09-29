//! App-icon rendering for the shipped set (design/08-ICONS.md 2, 3.7).
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
mod font;
mod grain;
mod mark;
mod plane;
mod plate;
mod retint;
mod sheet;
mod ship;
mod spec;
mod template;

pub use bevel::{finish, plate_face};
pub use board::render_icon;
pub use color::{Oklab, Srgb8, linear_to_srgb, oklab, srgb, srgb_to_linear};
pub use compose::{drop_shadow, over};
pub use dialect::{CHROMA_CAP, ChromaCap, Dialect, Lch, PALETTE, Roles, Shift, Tint, roles};
pub use emblem::{EMBOSS_FROM, Look, emblem, emblem_object};
pub use error::IconsError;
pub use export::grid_for;
pub use font::{draw_text, text_width};
pub use grain::{GRAIN_TILE, apply_grain, grain_tile};
pub use mark::{BAND, Corner, Corners, Heading, Pt, Shape, distance};
pub use plane::Plane;
pub use plate::{PlateGrid, plate_mask};
pub use retint::retint;
pub use sheet::{Cell, Sheet, SheetStyle, build_sheet, strip};
pub use ship::{
    FACE_CROP, Manifest, Prepared, SHIP_PX, ShipApp, Source, Style, face_icon, klein_face,
    ship_icon, style_look,
};
pub use spec::{Grain, Layer, Relief, Role, Spec, parse_spec};
pub use template::{Fraction, Template};
