//! The root: `div.ds` carrying `data-theme`, `data-accent`, `data-motion`, `data-material`,
//! `data-blur`, `data-modality` and the hover hub's `data-hover`, with the frame's `--f-*`
//! inline; then the stylesheet (when inlined), the frame layers and grain, the children, the
//! overlay host and the toast host. It provides `Env`, `HoverHub`, `ToastHub`, `LayerStack`
//! and `Overlays` as context. Its click handler, the last to hear a click, hands a click that
//! landed on nothing focusable to the host's `HostClickFocus` (FINDINGS "Native focus").
//!
//! What the root paints follows its material (`chrome.rs`): a Window draws the Space gradient
//! opaque with its A/B layers and grain (`data-frame="opaque"`, its own stacking context so the
//! layers paint over its background); the bar, the dock, a popover panel, the OSD and a
//! widget draw the same layers and grain as one group at the material's tint alpha
//! (`data-frame="tinted"`, design/21-SPACES.md sections 3 and 5); a Popover, Sheet or Toast root
//! paints nothing on its own box (`data-chrome="transparent"`) and its cards paint the material;
//! the bar and the dock stamp `data-ground="frame"` so the components on them take the `--f-*`
//! inks.
//!
//! The root also writes `--m-tint-alpha` inline: the materials' tint over blur scales by the
//! `appearance.material_tint_alpha` settings key (design/22-SETTINGS.md section 3.1, default
//! 80; FINDINGS "W1 tokens"). A host passes the key as the `tint_alpha` prop (thousandths: 800
//! is .8), which `ds_settings::Environment::tint_alpha` converts from the settings file; without
//! one the root writes the key's default. A `radius` overrides the material's corner the same way
//! (`--m-radius` inline): the dock's pill is its root, and its radius is `dock.pill_radius_px`
//! (sill FINDINGS Q15). A `stack` writes the material stack's settings (`MaterialStack`: the
//! highlight, hairline, shadow strength and vibrancy keys) the same way.
//!
//! The root also writes the pixel tokens' inputs for its device scale (`scale`, else the host's
//! `HostScale`, else 1x; `tokens/pixel.rs`), so every hairline is whole device pixels at 1.25,
//! 1.5 or 1.75 (design/01-LAYOUT.md section 2.1). At 1x it writes nothing.

use super::chrome::{FrameTint, Ground, RootChrome};
use super::env::{Env, HostModality, InputModality, use_env_provider};
use super::scale::use_root_scale;
use crate::appearance::{Appearance, SystemPrefs, resolve};
use crate::components::toast::ToastHost;
use crate::focus::click::{HostClickFocus, after_click};
use crate::geometry::Scale;
use crate::material::recipe::DEFAULT_TINT_ALPHA;
use crate::material::{BlurState, Material, MaterialStack};
use crate::overlay::host::{OverlayHost, use_overlays_provider};
use crate::overlay::hover_hub::{HoverWarmth, use_hover_hub_provider};
use crate::overlay::stack::LayerStack;
use crate::overlay::toast_hub::use_toast_hub_provider;
use crate::space::{FrameVars, SpaceLook};
use crate::tokens::hex::Alpha;
use crate::tokens::{Corner, PixelToken};
use dioxus::prelude::*;
use std::rc::Rc;

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
    #[props(default)] chrome: Option<RootChrome>,
    #[props(default)] ground: Option<Ground>,
    #[props(default)] frame: Option<FrameTint>,
    #[props(default)] radius: Option<Corner>,
    #[props(default)] stack: Option<MaterialStack>,
    #[props(default)] scale: Option<Scale>,
    children: Element,
) -> Element {
    let scale = use_root_scale(scale);
    let chrome = chrome.unwrap_or(RootChrome::of(material));
    let frame_tint = frame.unwrap_or(FrameTint::of(material, chrome));
    let ground = ground.unwrap_or(Ground::of(material));
    let resolved = resolve(appearance, look.theme, system);
    let host = use_hook(try_consume_context::<HostModality>);
    let click_focus = use_hook(try_consume_context::<HostClickFocus>);
    let mut element = use_hook(|| CopyValue::new(None::<Rc<MountedData>>));
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
    let corner = radius.map(super::surface::radius_style).unwrap_or_default();
    let stack = stack.map(|stack| stack.style_attr()).unwrap_or_default();
    let pixels = PixelToken::style_attr(scale);
    let style = format!(
        "{}--m-tint-alpha:{};{corner}{stack}{pixels}",
        frame.style_attr(),
        tint.css()
    );
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
            "data-chrome": chrome.attribute(),
            "data-frame": frame_tint.attribute(),
            "data-ground": ground.attribute(),
            "data-corner": radius.and_then(Corner::attribute),
            style,
            onmounted: move |event: MountedEvent| element.set(Some(event.data())),
            // Last to hear a click: where the keyboard goes when it landed on nothing focusable.
            onclick: move |event: MouseEvent| after_click(click_focus, element.peek().clone(), &event),
            if stylesheet == Inject::Inline {
                style { {crate::css::stylesheet()} }
            }
            match frame_tint {
                FrameTint::Opaque => frame_layers(layers),
                FrameTint::Tinted => rsx! {
                    div { class: "ds-frame", {frame_layers(layers)} }
                },
                FrameTint::None => rsx! {},
            }
            {children}
            OverlayHost {}
            ToastHost {}
        }
    }
}

/// The two gradient layers and the grain over them.
fn frame_layers(layers: [(&'static str, Option<&'static str>, String); 2]) -> Element {
    rsx! {
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
