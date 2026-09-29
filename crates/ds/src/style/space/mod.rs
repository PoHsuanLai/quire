//! A Space's colours: the dots a person places, the palette derived from them, the frame
//! variables a `.ds` root carries, and the contrast arithmetic that keeps them legible
//! (design/03-COLOR.md sections 4-8, design/21-SPACES.md).

pub(crate) mod dot_paint;
pub(crate) mod frame_vars;
pub(crate) mod look;
pub(crate) mod palette;
pub(crate) mod presets;
pub(crate) mod store;
