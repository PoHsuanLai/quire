//! Colour spaces as types (8-bit sRGB, linear light, OKLab, OKLCh) and the conversions between
//! them, WCAG contrast, and the prototype's gamut fit. Every colour computation in the crate
//! converts through these.

pub(crate) mod contrast;
pub(crate) mod fit;
pub(crate) mod oklab;
pub(crate) mod srgb;
