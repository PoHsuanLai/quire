//! The appearance vocabulary: what a person picks (theme, accent, motion, look), what the
//! desktop says (system preferences), and what the two resolve to on a `.ds` root; and which
//! typeface the root's type speaks in.

pub(crate) mod accent;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub(crate) mod appearance;
pub(crate) mod blur;
pub(crate) mod look;
pub(crate) mod material;
pub(crate) mod motion;
pub(crate) mod peek;
pub(crate) mod resolve;
pub(crate) mod system;
pub(crate) mod theme;
pub(crate) mod typeface;
