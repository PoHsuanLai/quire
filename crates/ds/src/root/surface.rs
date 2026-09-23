//! A nested scope: a subtree drawn in another material, or another scheme, under the same root.
//! No stylesheet of its own.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::appearance::Scheme;
use crate::material::Material;
use dioxus::prelude::*;

/// A subtree in `material`, optionally forcing `theme`.
#[component]
pub fn Surface(
    material: Material,
    #[props(default)] theme: Option<Scheme>,
    children: Element,
) -> Element {
    todo!()
}
