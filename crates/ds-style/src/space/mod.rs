//! A Space's colours: the dots a person places, the palette derived from them, the frame
//! variables a `.ds` root carries, and the contrast arithmetic that keeps them legible
//! (design/03-COLOR.md sections 4-8, design/21-SPACES.md).

pub mod dot_paint;
pub mod frame_vars;
pub mod list;
pub mod look;
pub mod look_json;
pub mod palette;
pub mod presets;
pub mod store;
