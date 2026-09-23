//! Each material's tint, edge, shadow and radius per scheme: the `--m-*` tokens
//! (design/03-COLOR.md section 17.2, every value proposed, tuned in the gallery).
//!
//! The tint alpha over blur is a settings key (`appearance.material_tint_alpha`, default 80),
//! so the recipe takes it rather than hard-coding `.80`.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::material::Material;
use crate::appearance::Scheme;
use crate::tokens::hex::Alpha;

/// The five `--m-*` values one material paints in one scheme, as CSS values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRecipe {
    /// `--m-tint`: the translucent tint over blur.
    pub tint: String,
    /// `--m-tint-solid`: the same tint at alpha .94 or above, when blur is unavailable.
    pub tint_solid: String,
    /// `--m-edge`: hairline and highlight.
    pub edge: String,
    /// `--m-shadow`: the drop shadow, or `none`.
    pub shadow: String,
    /// `--m-radius`.
    pub radius: String,
}

/// What `material` paints in `scheme`, with its tint at `tint_alpha` over blur.
pub fn recipe(material: Material, scheme: Scheme, tint_alpha: Alpha) -> MaterialRecipe {
    todo!()
}
