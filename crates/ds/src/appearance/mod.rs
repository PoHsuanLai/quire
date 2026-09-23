//! The appearance vocabulary: what a person picks (theme, accent, motion, look), what the
//! desktop says (system preferences), and what the two resolve to on a `.ds` root.

pub mod accent;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub mod appearance;
pub mod look;
pub mod motion;
pub mod peek;
pub mod resolve;
pub mod system;
pub mod theme;

pub use accent::Accent;
pub use appearance::Appearance;
pub use look::{Look, Warmth};
pub use motion::{Motion, MotionLevel};
pub use peek::PeekMode;
pub use resolve::{Resolved, resolve};
pub use system::{Contrast, ReducedMotion, SystemPrefs};
pub use theme::{Scheme, Theme};
