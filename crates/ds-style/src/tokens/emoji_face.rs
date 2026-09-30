//! `--font-emoji`: the colour emoji face first, then the text face (FINDINGS "Colour emoji").
//!
//! With no emoji family named, fontique's fallback on the reference machine picks Symbola, a
//! monochrome face that also lacks skin tones, flags and ZWJ sequences; Noto Color Emoji
//! (COLRv1, installed by default on Fedora) paints them all in colour on vello_cpu and
//! vello_hybrid. Inter leads and the emoji face follows: Noto Color Emoji maps the digits, `#`
//! and `*` (the bases of keycap sequences) to empty glyphs, so with the emoji face first "2#"
//! drew nothing (measured; tests/colour_emoji.rs in ds-blitz keeps the text pixel-equal to
//! Inter's). Every emoji Inter lacks, which is all the pictographs, flags, skin tones and ZWJ
//! sequences, falls through to the colour face. Consumers may not write a `font-family` themselves (the lint), so quire carries
//! the stack, and `.ds-emoji-text` applies it. Kept apart from [`super::Family`], whose members are
//! the faces quire ships; this one is the system's.

use super::name::VarName;
use super::token::Token;
use ds_core::word::Word;

/// The emoji face's custom property: the one member of its family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "font-", kind = fixed)]
pub enum EmojiFace {
    /// `--font-emoji`: the text face, then the colour emoji face.
    #[token(value = "\"Inter\",\"Noto Color Emoji\",system-ui,sans-serif")]
    Emoji,
}

/// The custom property the emoji stack is declared as.
pub const FONT_EMOJI: VarName = EmojiFace::Emoji.var();
