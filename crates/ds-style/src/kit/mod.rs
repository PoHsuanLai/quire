//! The kit seam: what each layer adds to the stylesheet and to the linter's vocabulary, and how
//! the layers are ordered. The cascade's order is each kit's rank, never the order kits are
//! listed in.

mod blocks;
mod custom_styles;
mod kits;
mod model;
pub mod reach;
mod style_kit;
mod styles;
#[cfg(test)]
mod tests;
mod user_style;

pub use custom_styles::CustomStyles;
pub use kits::{Kits, KnownNames};
pub use model::{Kit, KitRank, Section, Sheet, Vocabulary};
pub use styles::{Drawn, Dropped, Fingerprint, LookStyle, Slot, StyleHomes, Styles};
pub use user_style::UserStyle;

pub use style_kit::KIT as STYLE_KIT;
