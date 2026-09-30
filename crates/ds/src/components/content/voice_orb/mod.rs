//! The voice orb: a round, softly turning field of colour that shows a voice assistant is
//! listening (design/30 section 2.9).

pub(crate) mod io;
pub mod model;
pub(crate) mod step;
#[cfg(test)]
mod tests;
pub mod view;
