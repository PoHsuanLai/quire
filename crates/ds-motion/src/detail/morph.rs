//! How one glyph gives way to the next (design/26-DETAILS.md section 3.2, A5).

use ds_core::word::Word;

/// The morph's component sheet, appended to the stylesheet by the assembly in its cascade slot.
pub const CSS: &str = include_str!("morph.css");

/// How a glyph morphs into the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum MorphStyle {
    /// Opacity only: two glyphs of one shape family (the Wi-Fi strengths, battery buckets).
    CrossFade,
    /// The base stays (cross-fading if it changes); a slash draws on or off (mute, radio off).
    Slash,
}

/// Whether a `MorphStyle::Slash` glyph wears its slash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Slashed {
    /// No slash: the thing is on.
    #[default]
    Off,
    /// Slashed: muted, off.
    On,
}
