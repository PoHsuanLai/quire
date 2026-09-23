//! A nested scope: a subtree drawn in another material, or another scheme, under the same root.
//! No stylesheet of its own.

use super::env::{Env, use_env, use_env_provider};
use crate::appearance::Scheme;
use crate::material::Material;
use dioxus::prelude::*;

/// A subtree in `material`, optionally forcing `theme`: a nested `div.ds` stamping the
/// scope's theme, accent, motion, material and blur, with no stylesheet and no frame
/// variables (it inherits the root's).
#[component]
pub fn Surface(
    material: Material,
    #[props(default)] theme: Option<Scheme>,
    children: Element,
) -> Element {
    let parent = use_env();
    let env = Env {
        scheme: theme.unwrap_or(parent.scheme),
        material,
        ..parent
    };
    use_env_provider(env);
    rsx! {
        div {
            class: "ds",
            "data-theme": env.scheme.slug(),
            "data-accent": env.resolved.accent.slug(),
            "data-motion": env.resolved.motion.slug(),
            "data-material": env.material.slug(),
            "data-blur": env.blur.slug(),
            {children}
        }
    }
}
