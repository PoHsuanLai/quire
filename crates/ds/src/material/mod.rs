//! Materials: what a shell surface is drawn in, tint, edge, shadow and radius over optional
//! compositor blur (design/03-COLOR.md section 17).
//!
//! The blur region's geometry belongs to shell-host, which owns the surface; ds exposes only
//! [`Material::blur`], the intent. (The plan's `blur_region()` was dropped from ds.)

pub mod blur;
#[allow(clippy::module_inception)] // The layout names the file for its one concept.
pub mod material;
pub mod recipe;

pub use blur::{Blur, BlurState};
pub use material::Material;
pub use recipe::{MaterialRecipe, recipe};
