//! The app-facing parts that sit above `ds`: the account sheets and the consent alert, the
//! missing-helper sheet, the confirm card, with their sheets, which `kits()` and `stylesheet()` add
//! to the design system's. Every part is cross-platform: an app takes this crate for them. The
//! surfaces only our desktop shell draws (bar, dock, control center, lock, switcher, OSD,
//! notifications, widgets and the rest) live in sill's `sill-shell-kit`.

pub mod accounts;
pub mod confirm;
pub mod helpers;
pub(crate) mod kit;
pub(crate) mod sheets;
#[cfg(test)]
mod vocabulary_tests;

pub mod prelude;

// The curated roots: the stylesheet assembly. Every other name is in `prelude` or at its home path.
pub use crate::kit::{KIT, component_sheets, kits, stylesheet};
