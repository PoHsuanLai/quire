//! The token table: every colour, duration, delay, easing, scalar, radius, spacing step,
//! shadow, type size, layer and device-pixel line width, as Rust data. Each family is a `Token`
//! (`#[derive(Word, Token)]`) listed in a kit, and the stylesheet's `.ds` blocks and the linter's
//! vocabulary are written from those lists (`crate::style::kit`), so CSS and the Rust timers
//! cannot drift (design/05-MOTION.md section 7.1).

pub(crate) mod accent_band;
pub(crate) mod accent_table;
pub(crate) mod colour;
pub(crate) mod control_size;
pub(crate) mod curve;
pub(crate) mod delay;
pub(crate) mod easing;
pub(crate) mod elevation;
pub(crate) mod emoji_face;
pub(crate) mod hex;
pub(crate) mod label_hue;
pub(crate) mod layer;
pub(crate) mod name;
pub(crate) mod opacity;
pub(crate) mod orb;
pub(crate) mod person;
pub(crate) mod pixel;
pub(crate) mod plate;
pub(crate) mod scalar;
pub(crate) mod set;
pub(crate) mod shape;
#[cfg(test)]
pub(crate) mod size_rules_tests;
pub(crate) mod size_scale;
pub(crate) mod size_vars;
pub(crate) mod spacing;
pub(crate) mod status;
pub(crate) mod timing;
pub(crate) mod tint;
pub(crate) mod token;
pub(crate) mod tuned;
pub(crate) mod type_scale;
pub(crate) mod type_voice;
pub(crate) mod vibrancy;
