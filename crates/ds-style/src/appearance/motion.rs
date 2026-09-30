//! How much a surface moves: the preference a person picks ([`Motion`]) and the level the
//! token table is written against ([`MotionLevel`]).
//!
//! Motion is an accessibility preference, not a style (design/30-CATALOGUE.md section 1.1):
//! [`Motion`] is `appearance.motion`, [`MotionLevel`] is what a root is drawn at, and `Reduced`
//! is macOS's "Reduce motion" with cross-fade transitions on.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// How much the window moves, as a person chooses it.
///
/// The desktop's own reduced-motion preference always wins over `Standard`
/// (`resolve`), so a person who asked the desktop to reduce motion is never moved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    /// The macOS defaults, unless the desktop asks for reduced motion.
    #[default]
    Standard,
    /// Slides, scales and springs become a cross-fade; nothing overshoots.
    Reduced,
}

/// The motion level a `.ds` root is drawn at: a [`Motion`] with the desktop's answer applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum MotionLevel {
    /// The tables of design/30 section 1.2 as written.
    #[default]
    Standard,
    /// Every moving duration is `--t-quick`, springs are critically damped, drags track 1:1.
    Reduced,
}
