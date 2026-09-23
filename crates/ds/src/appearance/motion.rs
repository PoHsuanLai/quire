//! How much a surface moves: the preference a person picks ([`Motion`]) and the level the
//! token table is written against ([`MotionLevel`]).
//!
//! [`Motion`] moved from mailo (`mail-app/src/view.rs`) and gained `System` and `Reduced`
//! (design/22-SETTINGS.md section 3.1 `appearance.motion_level`, default `System`).
//! [`MotionLevel`] is design/05-MOTION.md section 3.2: one attribute that rescales the whole
//! system, `Reduced` = 60 ms everywhere.

use serde::{Deserialize, Serialize};

/// How much the window moves, as a person chooses it.
///
/// One setting that rescales the whole motion system rather than a switch per animation.
/// `System` follows the desktop's reduced-motion preference; the other four name a level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
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

impl Motion {
    /// Every choice, in the order a picker offers them.
    pub const ALL: [Motion; 5] = [
        Motion::System,
        Motion::Calm,
        Motion::Standard,
        Motion::Extra,
        Motion::Reduced,
    ];

    /// The stored word.
    pub fn slug(self) -> &'static str {
        match self {
            Motion::System => "system",
            Motion::Calm => "calm",
            Motion::Standard => "standard",
            Motion::Extra => "extra",
            Motion::Reduced => "reduced",
        }
    }

    /// What a picker calls it.
    pub fn label(self) -> &'static str {
        match self {
            Motion::System => "System",
            Motion::Calm => "Calm",
            Motion::Standard => "Standard",
            Motion::Extra => "Extra",
            Motion::Reduced => "Reduced",
        }
    }

    /// The choice a stored word names, or [`None`] for a word that is not one.
    pub fn parse(word: &str) -> Option<Motion> {
        Self::ALL.into_iter().find(|level| level.slug() == word)
    }
}

/// The motion level a `.ds` root is drawn at: a [`Motion`] with `System` answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
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

impl MotionLevel {
    /// Every level, in the order the gallery's motion axis shows them.
    pub const ALL: [MotionLevel; 4] = [
        MotionLevel::Calm,
        MotionLevel::Standard,
        MotionLevel::Extra,
        MotionLevel::Reduced,
    ];

    /// The `data-motion` value on a `.ds` root. Always explicit, never omitted.
    pub fn slug(self) -> &'static str {
        match self {
            MotionLevel::Calm => "calm",
            MotionLevel::Standard => "standard",
            MotionLevel::Extra => "extra",
            MotionLevel::Reduced => "reduced",
        }
    }
}
