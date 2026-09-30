//! The appearance vocabulary: what a person picks (theme, accent, motion), what the
//! desktop says (system preferences), and what the two resolve to on a `.ds` root; and which
//! typeface the root's type speaks in.

pub mod accent;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub mod appearance;
pub mod blur;
pub mod material;
pub mod motion;
pub mod peek;
pub mod resolve;
pub mod system;
pub mod theme;
pub mod typeface;
