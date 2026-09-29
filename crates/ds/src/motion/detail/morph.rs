//! How one glyph gives way to the next (design/26-DETAILS.md section 3.2, A5).

/// How a glyph morphs into the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MorphStyle {
    /// The old glyph shrinks to .7 and fades as the new one grows from .7: a change of state.
    DownUp,
    /// The old glyph goes at once and the new one grows in: the next available action.
    OffUp,
    /// Opacity only: two glyphs of one shape family (the Wi-Fi strengths, battery buckets).
    CrossFade,
    /// The base stays (cross-fading if it changes); a slash draws on or off (mute, radio off).
    Slash,
}

impl MorphStyle {
    /// The `data-style` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            MorphStyle::DownUp => "down-up",
            MorphStyle::OffUp => "off-up",
            MorphStyle::CrossFade => "cross-fade",
            MorphStyle::Slash => "slash",
        }
    }
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
