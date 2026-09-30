//! The base layer of the design system: vocabulary, geometry, time, errors, colour and encodings,
//! plain data and maths with no renderer and no Dioxus, so a crate that only reads settings or
//! services can use it. `#[derive(Word)]` expands to paths in this crate alone.

// The derive names the trait `::ds_core::word::Word`, so inside this crate `ds_core` is the
// crate itself.
extern crate self as ds_core;

pub mod base64;
pub mod colour;
pub mod error;
pub mod geometry;
pub mod png;
pub mod press;
pub mod spawner;
pub mod standard_action;
pub mod text;
pub mod time;
pub mod vocab;
pub mod word;

#[cfg(any(test, feature = "testing"))]
pub mod testing;
