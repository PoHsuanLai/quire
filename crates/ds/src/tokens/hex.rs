//! Colour values as the token table holds them: an 8-bit sRGB hex, optionally with an alpha.
//!
//! No floats, so every colour is `Eq` and a token table can be compared in a test.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

/// An 8-bit sRGB colour, written `#rrggbb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex(pub [u8; 3]);

impl Hex {
    /// Parse `#rgb` or `#rrggbb`, any case; `None` for anything else.
    pub fn parse(text: &str) -> Option<Hex> {
        todo!()
    }

    /// Lower-case `#rrggbb`.
    pub fn css(self) -> String {
        todo!()
    }
}

/// Opacity in thousandths: 1000 is opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Alpha(pub u16);

/// A token's value: solid, or a colour at an alpha (`rgba(...)`), which is how the washes,
/// the scrim and the frame pills are written now that `color-mix()` is gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Colour {
    /// An opaque colour.
    Solid(Hex),
    /// A colour at an alpha.
    Alpha(Hex, Alpha),
}

impl Colour {
    /// The CSS text: `#rrggbb` or `rgba(r,g,b,.a)`.
    pub fn css(self) -> String {
        todo!()
    }
}
