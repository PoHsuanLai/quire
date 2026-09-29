//! The voice orb's data: its colours, how far round it has turned, and the look values that
//! follow from its size.

use crate::core::geometry::units::Px;
use crate::core::vocab::Percent;
use crate::style::tokens::hex::Hex;

/// One of the orb's four colours: the design token's value, which follows the scheme and the
/// Look, or a colour the caller brings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OrbColour {
    /// The orb token (`--orb-bg`, `--orb-c1`, `--orb-c2`, `--orb-c3`) for this colour.
    #[default]
    Token,
    /// A colour of the caller's own.
    Custom(Hex),
}

/// The orb's colours: the ground the glows fade to, and the three glows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct OrbColours {
    /// The ground: the rim's ring and the dots (`--orb-bg`).
    pub bg: OrbColour,
    /// The first glow (`--orb-c1`).
    pub c1: OrbColour,
    /// The second glow (`--orb-c2`).
    pub c2: OrbColour,
    /// The third glow (`--orb-c3`).
    pub c3: OrbColour,
}

/// How far round the glows have turned, in millionths of a full turn: 0 is where they start and
/// the same place as a whole turn, so it wraps at [`Turn::FULL`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Turn(pub u32);

impl Turn {
    /// One whole turn.
    pub const FULL: u32 = 1_000_000;

    /// The angle in degrees (a float because CSS angles are).
    pub fn degrees(self) -> f32 {
        self.0 as f32 * 360.0 / Self::FULL as f32
    }
}

/// A contrast amount as CSS `contrast()` reads it: 1 leaves colours alone.
///
/// Holds a float because it is derived from a [`Px`], which is one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contrast(pub f32);

/// The mask that fades the dots out toward the rim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrbMask {
    /// No mask: a small orb shows its dots evenly.
    Off,
    /// The dots are fully there inside this share of the radius and fade out by 75 %.
    Radius(Percent),
}

/// What the orb's size sets: how soft, how contrasty, how fine its dots, how wide its rim.
///
/// Holds floats because every value is a [`Px`] or a [`Contrast`] scaled from the orb's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbMetrics {
    /// The glow layer's blur radius.
    pub blur: Px,
    /// The glow layer's contrast.
    pub contrast: Contrast,
    /// The radius of a dot in the dot grid; the grid's cell is twice it.
    pub dot: Px,
    /// The rim's soft shadow spread.
    pub shadow: Px,
    /// The dot mask.
    pub mask: OrbMask,
}

impl OrbMetrics {
    /// The values for an orb `size` across: an orb under 50 px takes finer, gentler values
    /// than a larger one, and under 30 px it drops the dot mask and takes the lowest contrast.
    pub fn of(size: Px) -> Self {
        let s = size.0;
        let small = s < 50.0;
        let pick = |fine: (f32, f32), large: (f32, f32)| {
            let (scale, floor) = if small { fine } else { large };
            (s * scale).max(floor)
        };
        let base_contrast = pick((0.004, 1.2), (0.008, 1.5));
        let contrast = if s < 30.0 {
            1.1
        } else if small {
            (base_contrast * 1.2).max(1.3)
        } else {
            base_contrast
        };
        let mask = match s {
            s if s < 30.0 => OrbMask::Off,
            s if s < 50.0 => OrbMask::Radius(Percent(5)),
            s if s < 100.0 => OrbMask::Radius(Percent(15)),
            _ => OrbMask::Radius(Percent(25)),
        };
        OrbMetrics {
            blur: Px(pick((0.008, 1.0), (0.015, 4.0))),
            contrast: Contrast(contrast),
            dot: Px(pick((0.004, 0.05), (0.008, 0.1))),
            shadow: Px(pick((0.004, 0.5), (0.008, 2.0))),
            mask,
        }
    }
}
