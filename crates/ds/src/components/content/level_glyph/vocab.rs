//! The level glyph's vocabulary: which glyph a level carries and where it takes its state from.
//! Every choice is a named variant (no `bool`, CONVENTIONS section 4).
use crate::components::content::status::volume::VolumeState;
use ds_core::vocab::{Fraction, Muting};

/// The glyph a level carries, which follows the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelGlyph {
    /// A speaker: no wave at 0, then one, two and three waves by thirds; muted, a slash.
    Volume(Muting),
    /// A sun whose rays grow with the brightness.
    Brightness,
    /// A keyboard under a rising sun whose rays grow with the keyboard's backlight (design/26
    /// 5.2.7): the keyboard-brightness module.
    KeyboardBrightness,
}

/// Where a level's glyph takes its state from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelSource {
    /// A glyph that follows the control's own value (a `LevelGlyph` converts).
    Glyph(LevelGlyph),
    /// The volume item's state: the speaker shows the waves and the slash that the bar's
    /// `VolumeGlyph` shows for the same state, so the Sound module and the bar read one source
    ///. The capsule still shows the control's value; a level of 0 with `Muted`
    /// keeps the slash.
    Volume(VolumeState),
}

impl LevelSource {
    /// The glyph drawn, and the level its parts follow, with the control at `value`.
    pub(crate) fn drawn(self, value: Fraction) -> (LevelGlyph, Fraction) {
        match self {
            LevelSource::Glyph(glyph) => (glyph, value),
            LevelSource::Volume(state) => state.glyph(),
        }
    }
}

/// Marks the two conversions below, so they do not collide with dioxus's own.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelSlot;

/// `Slider { glyph: LevelGlyph::Brightness }`: a level glyph fills a slot that takes a
/// [`LevelSource`].
impl dioxus::core::SuperFrom<LevelGlyph, LevelSlot> for Option<LevelSource> {
    fn super_from(glyph: LevelGlyph) -> Self {
        Some(LevelSource::Glyph(glyph))
    }
}

/// `Slider { glyph: volume_state }` too.
impl dioxus::core::SuperFrom<VolumeState, LevelSlot> for Option<LevelSource> {
    fn super_from(state: VolumeState) -> Self {
        Some(LevelSource::Volume(state))
    }
}

impl From<LevelGlyph> for LevelSource {
    fn from(glyph: LevelGlyph) -> Self {
        LevelSource::Glyph(glyph)
    }
}

impl From<VolumeState> for LevelSource {
    fn from(state: VolumeState) -> Self {
        LevelSource::Volume(state)
    }
}
