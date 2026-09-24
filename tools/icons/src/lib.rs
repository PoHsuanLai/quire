//! App-icon post-processing for the bake-off and, later, the set (design/08-ICONS.md 2, 3.7).
//!
//! Every function here is pure: images in, images out. Only `main.rs` reads and writes files.

mod color;
mod compose;
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
mod sheet;
mod spec;
mod template;

pub use color::{Oklab, Srgb8, delta_e, linear_to_srgb, oklab, srgb_to_linear};
pub use compose::{Shadow, compose, compose_on, drop_shadow, over};
pub use emblem::{emblem, emblem_object};
pub use error::IconsError;
pub use export::{EXPORT_SIZES, Export, export, grid_for};
pub use family::{Family, Stops};
pub use fit::{Rect, alpha_bbox, fit_object};
pub use font::{draw_text, text_width};
pub use grain::{GRAIN_TILE, apply_grain, grain_tile};
pub use key::{Key, key_background};
pub use mark::{Corner, Corners, Heading, Outline, Pt, STROKE, Shape, distance};
pub use plane::Plane;
pub use plate::{PlateGrid, gradient, plate_mask};
pub use sheet::{Cell, Sheet, SheetStyle, build_sheet, size_strip, strip, strip_sheet};
pub use spec::{Grain, Layer, Lift, Paint, Spec, paint, parse_spec};
pub use template::{Fraction, Template};
