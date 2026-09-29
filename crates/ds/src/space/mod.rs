//! A Space's colours: the dots a person places, the palette derived from them, the frame
//! variables a `.ds` root carries, and the contrast arithmetic that keeps them legible
//! (design/03-COLOR.md sections 4-8, design/21-SPACES.md).

pub(crate) mod dot_paint;
pub mod frame_vars;
pub mod look;
pub mod palette;
pub mod presets;
pub mod store;

pub use crate::core::colour::contrast::{Verdict, ratio};
pub use crate::space::palette::{
    card::{Card, POST_DARK, POST_LIGHT, card},
    readout::{ContrastCheck, readout},
};
pub use frame_vars::FrameVars;
pub use look::{CardAccent, Grain, SpaceLook};
pub use palette::{Capping, Dot, NEUTRAL_DOT, Palette, derive, gradient, swatch};
pub use presets::{PRESETS, Preset, default_look};
pub use store::{SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex};
