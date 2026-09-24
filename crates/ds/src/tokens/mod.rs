//! The token table: every colour, duration, delay, easing, scalar, radius, spacing step,
//! shadow, type size and layer, as Rust data. The stylesheet is generated from it (`crate::css`), so CSS and the
//! Rust timers cannot drift (design/05-MOTION.md section 7.1).

pub mod accent_table;
pub mod colour;
pub mod delay;
pub mod easing;
pub mod elevation;
pub mod hex;
pub mod label_hue;
pub mod layer;
pub mod name;
pub mod scalar;
pub mod shape;
pub mod spacing;
pub mod timing;
pub mod type_scale;

pub use accent_table::{AccentQuad, quad};
pub use colour::ColourToken;
pub use delay::DelayToken;
pub use easing::{CubicBezier, Easing, EasingToken};
pub use elevation::Shadow;
pub use hex::{Alpha, Colour, Hex};
pub use label_hue::{HueMember, LabelHue};
pub use layer::ZLayer;
pub use name::VarName;
pub use scalar::{ScalarToken, ScalarValue};
pub use shape::{Corner, Radius};
pub use spacing::SpacingToken;
pub use timing::{DurationKind, DurationToken};
pub use type_scale::{Family, FontSize};
