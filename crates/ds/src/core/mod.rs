//! The base layer: vocabulary, geometry, time, tasks, errors, colour and encodings. It names
//! nothing else in the crate.

pub mod base64;
pub mod busy;
pub mod colour;
pub mod error;
pub mod geometry;
pub mod guarded;
pub mod png;
pub mod press;
pub mod spawner;
pub(crate) mod standard_action;
pub mod task;
pub mod text;
pub mod time;
pub mod vocab;
pub mod word;
