//! Materials: what a shell surface is drawn in, tint, edge, shadow and radius over optional
//! compositor blur (design/03-COLOR.md section 17).
//!
//! The blur region's geometry belongs to shell-host, which owns the surface; ds exposes only
//! [`Material::blur`], the intent. (The plan's `blur_region()` was dropped from ds.)

pub(crate) mod layer;
pub(crate) mod level;

pub(crate) mod recipe;
pub(crate) mod stack;
