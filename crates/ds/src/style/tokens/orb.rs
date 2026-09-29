//! The custom properties the voice orb writes inline for its stylesheet to read: its size, the
//! size-derived look values and the turn (design/30 section 2.9). The four colours are the
//! `--orb-*` colour tokens ([`super::colour::ColourToken`]), written inline only when the caller
//! brings its own.

use super::name::VarName;

/// The orb's width and height.
pub const SIZE: VarName = VarName("--orb-size");
/// The glow layer's blur radius.
pub const BLUR: VarName = VarName("--orb-blur");
/// The glow layer's contrast.
pub const CONTRAST: VarName = VarName("--orb-contrast");
/// The radius of a dot in the dot grid.
pub const DOT: VarName = VarName("--orb-dot");
/// The rim's soft shadow spread.
pub const SHADOW: VarName = VarName("--orb-shadow");
/// The share of the radius inside which the dots are fully there.
pub const MASK: VarName = VarName("--orb-mask");
/// How far round the glows have turned, as an angle.
pub const TURN: VarName = VarName("--orb-turn");

/// Every one, for the lint's list of declared properties.
pub(crate) const ORB_VARS: [VarName; 7] = [SIZE, BLUR, CONTRAST, DOT, SHADOW, MASK, TURN];
