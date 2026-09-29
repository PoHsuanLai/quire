//! A nested scope: a subtree drawn in another material, scheme, accent or blur state, under the
//! same root. No stylesheet of its own.

use super::chrome::{Ground, RootChrome};
use super::env::{Env, use_env, use_env_provider};
use super::typeface::use_typeface;
use crate::appearance::{accent::Accent, resolve::Resolved, theme::Scheme};
use crate::material::{blur::BlurState, material::Material};
use crate::tokens::accent_band::text_grounds::{TextOn, text_on};
use crate::tokens::shape::Corner;
use dioxus::prelude::*;

/// A subtree in `material`, optionally forcing `theme`, `accent` or `blur`: a nested `div.ds`
/// stamping the scope's theme, the root's typeface, and the scope's accent, motion, material and
/// blur, with no stylesheet and no frame variables (it inherits the root's). Each override left
/// `None` inherits the enclosing scope's value, so a specimen in another accent or blur state
/// needs no second `Ds`. `on` is
/// the ground its content is drawn on: `None` takes the material's (`Ground::of`: the frame for
/// Bar and Dock, paper otherwise), so a paper panel inside a bar root is paper again. `radius`
/// overrides the material's corner (`--m-radius`), for a surface whose radius is a setting.
///
/// `chrome: Some(RootChrome::Transparent)` (notification parts) paints nothing on the scope's own
/// box and lets the cards inside it paint the material instead (`data-chrome="transparent"`, the
/// rule a transparent root follows): a notification card in the Toast material inside a
/// Popover-rooted center, without a Toast root of its own. `None` and `Painted` paint the box, as
/// before.
#[component]
pub fn Surface(
    material: Material,
    #[props(default)] theme: Option<Scheme>,
    #[props(default)] accent: Option<Accent>,
    #[props(default)] blur: Option<BlurState>,
    #[props(default)] on: Option<Ground>,
    #[props(default)] radius: Option<Corner>,
    #[props(default)] chrome: Option<RootChrome>,
    children: Element,
) -> Element {
    let env = scope(use_env(), material, theme, accent, blur);
    let ground = on.unwrap_or(Ground::of(material));
    let typeface = use_typeface();
    use_env_provider(env);
    rsx! {
        div {
            class: "ds",
            "data-theme": env.scheme.slug(),
            "data-typeface": typeface.slug(),
            "data-accent": env.resolved.accent.slug(),
            "data-motion": env.resolved.motion.slug(),
            "data-material": env.material.slug(),
            "data-blur": env.blur.slug(),
            "data-ground": ground.attribute(),
            "data-corner": radius.and_then(Corner::attribute),
            "data-chrome": chrome.and_then(RootChrome::attribute),
            style: scope_style(env.material, radius),
            {children}
        }
    }
}

/// A transparent scope of `material` that also carries a quire class of its own (`class`), for a
/// component whose scope must be laid out itself: Blitz places an absolutely positioned box
/// against its parent, not its nearest positioned ancestor, so an edge panel's scope has to fill
/// the root for the panel inside it to (notification parts).
#[component]
pub(crate) fn ClassedScope(material: Material, class: &'static str, children: Element) -> Element {
    let env = scope(use_env(), material, None, None, None);
    let typeface = use_typeface();
    use_env_provider(env);
    rsx! {
        div {
            class: "ds {class}",
            "data-theme": env.scheme.slug(),
            "data-typeface": typeface.slug(),
            "data-accent": env.resolved.accent.slug(),
            "data-motion": env.resolved.motion.slug(),
            "data-material": env.material.slug(),
            "data-blur": env.blur.slug(),
            "data-ground": Ground::of(material).attribute(),
            "data-chrome": RootChrome::Transparent.attribute(),
            style: scope_style(env.material, None),
            {children}
        }
    }
}

/// The inline declaration that points `--accent-text` at `--accent-text-material` on a root of a
/// translucent material that carries accent text (design/03-COLOR.md section 20.6), or nothing.
/// Inline, so it follows a Space's own inline accent and beats the `data-accent` blocks.
pub(crate) fn accent_text_style(material: Material) -> &'static str {
    match text_on(material) {
        TextOn::Card => "",
        TextOn::Material => "--accent-text:var(--accent-text-material);",
    }
}

/// A nested scope's inline style: its accent text and its corner, or none at all.
fn scope_style(material: Material, radius: Option<Corner>) -> Option<String> {
    let style = format!(
        "{}{}",
        accent_text_style(material),
        radius.map(radius_style).unwrap_or_default()
    );
    (!style.is_empty()).then_some(style)
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
