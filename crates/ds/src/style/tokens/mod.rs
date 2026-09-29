//! The token table: every colour, duration, delay, easing, scalar, radius, spacing step,
//! shadow, type size, layer and device-pixel line width, as Rust data. Each family is a `Token`
//! (`#[derive(Word, Token)]`) listed in a kit, and the stylesheet's `.ds` blocks and the linter's
//! vocabulary are written from those lists (`crate::style::kit`), so CSS and the Rust timers
//! cannot drift (design/05-MOTION.md section 7.1).

pub mod accent_band;
pub(crate) mod accent_table;
pub mod colour;
pub mod control_size;
pub(crate) mod curve;
pub mod delay;
pub mod easing;
pub(crate) mod elevation;
pub mod emoji_face;
pub mod hex;
pub mod label_hue;
pub mod layer;
pub(crate) mod name;
pub(crate) mod opacity;
pub mod orb;
pub(crate) mod person;
pub mod pixel;
pub(crate) mod plate;
pub mod scalar;
pub mod set;
pub mod shape;
#[cfg(test)]
pub mod size_rules_tests;
pub mod size_scale;
pub(crate) mod size_vars;
pub mod spacing;
pub(crate) mod status;
pub mod timing;
pub(crate) mod tint;
pub mod token;
pub mod tuned;
pub mod type_scale;
pub(crate) mod type_voice;
pub(crate) mod vibrancy;
