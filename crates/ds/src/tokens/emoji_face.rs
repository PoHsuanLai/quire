//! `--font-emoji`: the colour emoji face first, then the text face (FINDINGS "Colour emoji").
//!
//! With no emoji family named, fontique's fallback on the reference machine picks Symbola, a
//! monochrome face that also lacks skin tones, flags and ZWJ sequences; Noto Color Emoji
//! (COLRv1, installed by default on Fedora) paints them all in colour on vello_cpu and
//! vello_hybrid. Inter leads and the emoji face follows: Noto Color Emoji maps the digits, `#`
//! and `*` (the bases of keycap sequences) to empty glyphs, so with the emoji face first "2#"
//! drew nothing (measured; tests/colour_emoji.rs in ds-native keeps the text pixel-equal to
//! Inter's). Every emoji Inter lacks, which is all the pictographs, flags, skin tones and ZWJ
//! sequences, falls through to the colour face. Consumers may not write a `font-family` themselves (the lint), so quire carries
//! the stack, and `.ds-emoji-text` applies it. Kept apart from [`super::Family`], whose members are
//! the faces quire ships; this one is the system's.

use super::name::VarName;

/// The emoji stack's custom property.
pub const FONT_EMOJI: VarName = VarName("--font-emoji");

/// Its value: Inter, then the COLRv1 colour emoji face, then the generic fallbacks.
pub const FONT_EMOJI_STACK: &str = "\"Inter\",\"Noto Color Emoji\",system-ui,sans-serif";
