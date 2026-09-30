//! The level control's vocabulary: what it is for, how it looks, and which glyph it
//! carries. Every choice is a named variant (no `bool`, CONVENTIONS section 4).

use crate::components::content::status::volume::VolumeState;
use ds_core::vocab::{Fraction, Muting};
use ds_core::word::Word;

/// Whether the control takes input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum LevelMode {
    /// A control: pointer and keys move it (`role="slider"`, focusable).
    #[default]
    Interactive,
    /// A read-only level (`role="progressbar"`, not focusable, no handlers): the OSD's.
    ReadOnly,
}

/// How the level is drawn: three looks for the user to choose between (FINDINGS "Level control
/// (2026-09-25)" has the references each follows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum LevelLook {
    /// A thick capsule whose fill is the material's bright ink, with the glyph inside at its left
    /// end, two-toned where the fill covers it (recommended).
    #[default]
    Capsule,
    /// The capsule with a separate round knob riding the fill's end, the glyph before it.
    CapsuleKnob,
    /// Sixteen rounded squares that fill in one by one, the glyph before them.
    Segments,
}

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
