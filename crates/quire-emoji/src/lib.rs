//! The colour emoji face of a machine, found and put ahead of the monochrome ones in a
//! [`fontique::Collection`] (FINDINGS "Colour emoji").
//!
//! Nothing here names a renderer or a toolkit: quire's `ds-blitz` and temor's terminal both hand
//! their font collection to [`prefer_colour`], so one detection decides the face for both. A
//! caller that needs only the font file, or the table directory of a font, has [`colrv1_emoji_file`]
//! and [`table_listed`].

mod face;
mod file;
mod sfnt;

pub use face::{EMOJI_FAMILY_ENV, EmojiChoice, Fonts, prefer_colour};
pub use file::colrv1_emoji_file;
pub use sfnt::table_listed;
