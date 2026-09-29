//! How much a surface moves: the preference a person picks ([`Motion`]) and the level the
//! token table is written against ([`MotionLevel`]).
//!
//! [`Motion`] is design/22-SETTINGS.md section 3.1 `appearance.motion_level`, default `System`.
//! [`MotionLevel`] is design/05-MOTION.md section 3.2: one attribute that rescales the whole
//! system, `Reduced` = 60 ms everywhere.

use crate::core::word::Word;
use serde::{Deserialize, Serialize};

/// How much the window moves, as a person chooses it.
///
/// One setting that rescales the whole motion system rather than a switch per animation.
/// `System` follows the desktop's reduced-motion preference; the other four name a level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    /// Standard, unless the desktop asks for reduced motion.
    #[default]
    System,
    /// No overshoot, no stagger, no tilt.
    Calm,
    /// The design as drawn.
    Standard,
    /// More spring, more stagger, more tilt.
    Extra,
    /// Every animation runs once, at 60 ms.
    Reduced,
}

/// The motion level a `.ds` root is drawn at: a [`Motion`] with `System` answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum MotionLevel {
    /// `--t-big` 300 ms, no spring, no overshoot, no stagger.
    Calm,
    /// The look's own values.
    #[default]
    Standard,
    /// `--t-big` 560 ms, a stronger spring, more stagger.
    Extra,
    /// Every duration 60 ms, iteration forced to one, scalars neutral.
    Reduced,
}
