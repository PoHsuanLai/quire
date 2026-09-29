//! Colour spaces as types (8-bit sRGB, linear light, OKLab, OKLCh) and the conversions between
//! them, WCAG contrast, and the prototype's gamut fit. Every colour computation in the crate
//! converts through these.

pub mod contrast;
pub mod fit;
pub mod oklab;
pub mod srgb;
