//! The root: `div.ds` carrying `data-theme`, `data-accent`, `data-motion`, `data-material`,
//! `data-blur`, `data-modality` and the hover hub's `data-hover`, with the frame's `--f-*`
//! inline; then the stylesheet (when inlined), the frame layers and grain on a Window, the
//! children, the overlay host and the toast host. It provides `Env`, `HoverHub`, `ToastHub`,
//! `LayerStack` and `Overlays` as context.
//!
//! The root also writes `--m-tint-alpha` inline: the materials' tint over blur scales by the
//! `appearance.material_tint_alpha` settings key (design/22-SETTINGS.md section 3.1, default
//! 80; FINDINGS "W1 tokens"). A host passes the key as the `tint_alpha` prop (thousandths: 800
//! is .8), which `ds_settings::Environment::tint_alpha` converts from the settings file; without
//! one the root writes the key's default.

use super::env::{Env, HostModality, InputModality, use_env_provider};
use crate::appearance::{Appearance, SystemPrefs, resolve};
use crate::components::toast::ToastHost;
use crate::material::recipe::DEFAULT_TINT_ALPHA;
use crate::material::{BlurState, Material};
use crate::overlay::host::{OverlayHost, use_overlays_provider};
use crate::overlay::hover_hub::{HoverWarmth, use_hover_hub_provider};
use crate::overlay::stack::LayerStack;
use crate::overlay::toast_hub::use_toast_hub_provider;
use crate::space::{FrameVars, SpaceLook};
use crate::tokens::hex::Alpha;
use dioxus::prelude::*;

/// How the stylesheet reaches the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Inject {
    /// A `<style>` inside `.ds` (spike S1).
    #[default]
    Inline,
    /// The host adds `ds::stylesheet()` to the document itself (the S1 fallback).
    Host,
}

/// The design system's root. Everything a surface draws goes inside one.
#[component]
pub fn Ds(
    appearance: Appearance,
    #[props(default)] system: SystemPrefs,
    #[props(default)] look: SpaceLook,
    material: Material,
    #[props(default)] blur: BlurState,
    #[props(default)] stylesheet: Inject,
    #[props(default)] tint_alpha: Option<Alpha>,
    children: Element,
) -> Element {
    let resolved = resolve(appearance, look.theme, system);
    let host = use_hook(try_consume_context::<HostModality>);
    let modality = host.map_or(InputModality::default(), |HostModality(current)| current());
    let env = use_env_provider(Env {
        resolved,
        scheme: resolved.scheme,
        material,
        blur,
        modality,
    });
    let hover = use_hover_hub_provider(env);
    use_toast_hub_provider(env);
    use_context_provider(|| Signal::new(LayerStack::default()));
    use_overlays_provider();
    let frame = FrameVars::of(&look, resolved.scheme);
    let layers = use_frame_layers(&frame.gradient);
    let tint = tint_alpha.unwrap_or(DEFAULT_TINT_ALPHA);
    let style = format!("{}--m-tint-alpha:{};", frame.style_attr(), tint.css());
    let hover = match hover.warmth() {
        HoverWarmth::Warm => "warm",
        HoverWarmth::Cold => "cold",
    };
    rsx! {
        div {
            class: "ds",
            "data-theme": resolved.scheme.slug(),
            "data-accent": resolved.accent.slug(),
            "data-motion": resolved.motion.slug(),
            "data-material": material.slug(),
            "data-blur": blur.slug(),
            "data-modality": modality.slug(),
            "data-hover": hover,
            style,
            if stylesheet == Inject::Inline {
                style { {crate::css::stylesheet()} }
            }
            if material == Material::Window {
                for (slot , data_layer , gradient) in layers {
                    div {
                        key: "{slot}",
                        class: "ds-layer",
                        "data-layer": data_layer,
                        style: "--f-grad:{gradient}",
                    }
                }
                div { class: "ds-grain" }
            }
            {children}
            OverlayHost {}
            ToastHost {}
        }
    }
}

/// Which of the two frame layers is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Front {
    A,
    B,
}

/// The two frame layers' gradients and which one is in front.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameLayers {
    a: String,
    b: String,
    front: Front,
}

impl FrameLayers {
    /// Show `gradient`: when it is new, the hidden layer takes it and comes to the front, so
    /// the stylesheet's opacity transition cross-fades the two (design/21-SPACES.md
    /// section 5).
    fn show(self, gradient: &str) -> Self {
        let showing = match self.front {
            Front::A => &self.a,
            Front::B => &self.b,
        };
        if showing == gradient {
            return self;
        }
        match self.front {
            Front::A => FrameLayers {
                b: gradient.to_owned(),
                front: Front::B,
                ..self
            },
            Front::B => FrameLayers {
                a: gradient.to_owned(),
                front: Front::A,
                ..self
            },
        }
    }

    /// Each layer's key, `data-layer` attribute and gradient, in a fixed order so the nodes
    /// persist. The hidden layer carries `data-layer="back"` (the attribute form the
    /// stylesheet's `.ds-layer[*|data-layer=back]{opacity:0}` rule matches, per FINDINGS
    /// "`Ds`'s own `Material::Window` frame layers do not match their own stylesheet rule");
    /// the front layer carries no `data-layer` at all, so it hits the lint's `DsInternals`
    /// attribute set the same way `data-theme` etc. do, rather than a class the stylesheet
    /// never targets.
    fn render(&self) -> [(&'static str, Option<&'static str>, String); 2] {
        let data_layer = |slot| {
            if slot == self.front {
                None
            } else {
                Some("back")
            }
        };
        [
            ("a", data_layer(Front::A), self.a.clone()),
            ("b", data_layer(Front::B), self.b.clone()),
        ]
    }
}

/// The frame layers for this render. Kept outside any signal: `Ds` re-renders whenever its
/// look changes, and nothing else reads them.
fn use_frame_layers(gradient: &str) -> [(&'static str, Option<&'static str>, String); 2] {
    let mut layers = use_hook(|| {
        CopyValue::new(FrameLayers {
            a: gradient.to_owned(),
            b: gradient.to_owned(),
            front: Front::A,
        })
    });
    let next = layers.peek().clone().show(gradient);
    let rendered = next.render();
    layers.set(next);
    rendered
}
