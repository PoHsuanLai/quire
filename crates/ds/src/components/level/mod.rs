//! The level control (the user's brief of 2026-09-25; sill FINDINGS Q74): `LevelControl`, its
//! vocabulary, the machine that drives it, the glyph that follows it and its three looks.

mod control;
pub(crate) mod glyph;
mod look;
pub(crate) mod machine;
pub mod vocab;

pub use control::LevelControl;
pub use vocab::{LevelGlyph, LevelLook, LevelMode, Muting, Tick};
