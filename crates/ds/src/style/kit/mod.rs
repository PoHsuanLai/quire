//! The kit seam: what each layer adds to the stylesheet and to the linter's vocabulary, and how
//! the layers are ordered. The cascade's order is each kit's rank, never the order kits are
//! listed in.

mod blocks;
mod kits;
mod model;
mod style_kit;
#[cfg(test)]
mod tests;

pub use kits::{Kits, KnownNames};
pub use model::{Kit, KitRank, Section, Vocabulary};

pub(crate) use style_kit::KIT as STYLE_KIT;
