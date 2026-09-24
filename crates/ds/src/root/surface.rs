//! A nested scope: a subtree drawn in another material, scheme, accent or blur state, under the
//! same root. No stylesheet of its own.

use super::chrome::Ground;
use super::env::{Env, use_env, use_env_provider};
use crate::appearance::{Accent, Resolved, Scheme};
use crate::material::{BlurState, Material};
use crate::tokens::Corner;
use dioxus::prelude::*;

/// A subtree in `material`, optionally forcing `theme`, `accent` or `blur`: a nested `div.ds`
/// stamping the scope's theme, accent, motion, material and blur, with no stylesheet and no
/// frame variables (it inherits the root's). Each override left `None` inherits the enclosing
/// scope's value, so a specimen in another accent or blur state needs no second `Ds`. `on` is
/// the ground its content is drawn on: `None` takes the material's (`Ground::of`: the frame for
/// Bar and Dock, paper otherwise), so a paper panel inside a bar root is paper again. `radius`
/// overrides the material's corner (`--m-radius`), for a surface whose radius is a setting.
#[component]
pub fn Surface(
    material: Material,
    #[props(default)] theme: Option<Scheme>,
    #[props(default)] accent: Option<Accent>,
    #[props(default)] blur: Option<BlurState>,
    #[props(default)] on: Option<Ground>,
    #[props(default)] radius: Option<Corner>,
    children: Element,
) -> Element {
    let env = scope(use_env(), material, theme, accent, blur);
    let ground = on.unwrap_or(Ground::of(material));
    use_env_provider(env);
    rsx! {
        div {
            class: "ds",
            "data-theme": env.scheme.slug(),
            "data-accent": env.resolved.accent.slug(),
            "data-motion": env.resolved.motion.slug(),
            "data-material": env.material.slug(),
            "data-blur": env.blur.slug(),
            "data-ground": ground.attribute(),
            "data-corner": radius.and_then(Corner::attribute),
            style: radius.map(radius_style),
            {children}
        }
    }
}

/// The inline declaration that overrides a material's corner.
pub(crate) fn radius_style(radius: Corner) -> String {
    format!("--m-radius:{};{}", radius.css(), radius.squircle_style())
}

/// The scope a `Surface` provides: the parent's, with each given override applied.
fn scope(
    parent: Env,
    material: Material,
    theme: Option<Scheme>,
    accent: Option<Accent>,
    blur: Option<BlurState>,
) -> Env {
    Env {
        scheme: theme.unwrap_or(parent.scheme),
        material,
        blur: blur.unwrap_or(parent.blur),
        resolved: Resolved {
            accent: accent.unwrap_or(parent.resolved.accent),
            ..parent.resolved
        },
        ..parent
    }
}
