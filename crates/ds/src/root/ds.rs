//! The root: `div.ds` carrying `data-theme`, `data-accent`, `data-motion`, `data-material`,
//! `data-blur`, `data-modality` and the hover hub's `data-hover`, with the frame's `--f-*`
//! inline; then the stylesheet (when inlined), the frame layers and grain on a Window, the
//! children, and the overlay host. It provides `Env`, `HoverHub`, `ToastHub`, `LayerStack` and
//! `Overlays` as context.
//!
//! `ToastHost` (components/toast.rs) is not rendered here yet: it is a wave 2 stub, and
//! rendering it would panic every root. It joins after `OverlayHost` when it is filled.

use super::env::{Env, HostModality, InputModality, use_env_provider};
use crate::appearance::{Appearance, SystemPrefs, resolve};
use crate::material::{BlurState, Material};
use crate::overlay::host::{OverlayHost, use_overlays_provider};
use crate::overlay::hover_hub::{HoverWarmth, use_hover_hub_provider};
use crate::overlay::stack::LayerStack;
use crate::overlay::toast_hub::use_toast_hub_provider;
use crate::space::{FrameVars, SpaceLook};
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
            style: frame.style_attr(),
            if stylesheet == Inject::Inline {
                style { {crate::css::stylesheet()} }
            }
            if material == Material::Window {
                for (slot , class , gradient) in layers {
                    div { key: "{slot}", class, style: "--f-grad:{gradient}" }
                }
                div { class: "ds-grain" }
            }
            {children}
            OverlayHost {}
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

    /// Each layer's key, class and gradient, in a fixed order so the nodes persist.
    fn render(&self) -> [(&'static str, &'static str, String); 2] {
        let class = |slot| {
            if slot == self.front {
                "ds-layer"
            } else {
                "ds-layer back"
            }
        };
        [
            ("a", class(Front::A), self.a.clone()),
            ("b", class(Front::B), self.b.clone()),
        ]
    }
}

/// The frame layers for this render. Kept outside any signal: `Ds` re-renders whenever its
/// look changes, and nothing else reads them.
fn use_frame_layers(gradient: &str) -> [(&'static str, &'static str, String); 2] {
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
