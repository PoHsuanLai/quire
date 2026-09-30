//! The root: `div.ds` carrying `data-theme`, `data-typeface`, `data-accent`, `data-motion`,
//! `data-material`, `data-blur`, `data-modality` and the hover hub's `data-hover`, with the frame's `--f-*`
//! inline; then the stylesheet (when inlined), the frame layers and grain, the children, the
//! overlay host and the toast host. It provides `Scope`, `HoverHub`, `ToastHub`, `LayerStack`
//! and `Overlays` as context. Its click handler, the last to hear a click, hands a click that
//! landed on nothing focusable to the host's `ClickFocusHost` (FINDINGS "Native focus"), and it
//! provides the same seams to the controls inside it, which hand over a click they keep to
//! themselves (`focus::click::kept_click`).
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
//! `user_style` is the person's own stylesheet (`ds_settings::UserStyle`, ARCHITECTURE.md section
//! 11): its text goes in `<style data-ds-user>` right after the design-system sheet, so a change
//! to the signal restyles the document and a rule of theirs wins by order. An empty style draws no
//! element at all.
//!
//! `surface` is the consumer's name for the surface this root is (`[data-surface=bar]`, the public
//! selector surface a user stylesheet targets, `selectors::SURFACE_ATTRIBUTE`).
//!
//! `stylesheet` says how the stylesheet reaches the document (`Inject`); `sheet` is the text
//! `Inject::Inline` writes: `None` is `ds::stylesheet()`, and a crate that adds components of its
//! own passes the sheet its kits make (`ds_shell::stylesheet()`), so the root draws them too.
//!
//! The root also writes `--m-tint-alpha` inline: the materials' tint over blur scales by the
//! `appearance.material_tint_alpha` settings key (design/22-SETTINGS.md section 3.1, default
//! 80). A host passes the key as the `tint_alpha` prop (thousandths: 800
//! is .8), which `ds_settings::Environment::tint_alpha` converts from the settings file; without
//! one the root writes the key's default. A `radius` overrides the material's corner the same way
//! (`--m-radius` inline): the dock's pill is its root, and its radius is `dock.pill_radius_px`
//!. A `stack` writes the material stack's settings (`MaterialStack`: the
//! highlight, hairline, shadow strength and vibrancy keys) the same way.
//!
//! The root also writes the pixel tokens' inputs for its device scale (`scale`, else the host's
//! `HostSignals`, else 1x; `tokens/pixel.rs`), so every hairline is whole device pixels at 1.25,
//! 1.5 or 1.75 (design/01-LAYOUT.md section 2.1). At 1x it writes nothing.
//!
//! A client-decorated window passes `window: WindowFrame::Titlebar { .. }`: the root stamps
//! `data-window-frame` and draws the titlebar, its children in `div.ds-window-body`, and the
//! resize edges (`components/window_frame.rs`). The default draws nothing more.
//! `typeface` picks the faces the family tokens name (design/02-TYPE.md section 2): System
//! (Inter), or `Typeface::Editorial`, mail's Bricolage, Karla and Space Mono. `None` (the
//! default) takes the enclosing root's, so a root nested in another speaks as it does, and a
//! top-level root is System.
//! The host reads it from `appearance.typeface` (`ds_settings::AppearanceSettings::typeface`), or
//! an app that keeps its own voice passes Editorial outright.
//! An overlay root passes `extent: RootExtent::Viewport` (`extent.rs`): a root holding only
//! positioned content (a centred sheet) is otherwise 0 px tall. A popup document, whose host
//! fits the surface to its content, passes `RootExtent::Popup`: the floating card lies in flow
//! at the origin and no outside catcher covers the surface.

use crate::components::chrome::window_frame::{WindowFrame, framed};
use crate::components::overlays::toast::ToastHost;
use crate::focus::click::ClickRoot;
use crate::host::signals::HostSignals;
use crate::root::chrome::{FrameTint, Ground, RootChrome};
use crate::root::extent::RootExtent;
use crate::root::typeface::{use_typeface, use_typeface_provider};
use crate::stack::host::{OverlayHost, use_overlays_provider};
use crate::stack::hover_hub::use_hover_hub_provider;
use crate::stack::layer_stack::LayerStack;
use crate::stack::toast_hub::use_toast_hub_provider;
use dioxus::prelude::*;
use ds_core::geometry::scale::Scale;
use ds_core::vocab::{Activity, InputModality};
use ds_core::word::Word;
use ds_motion::hover_intent::HoverWarmth;
use ds_style::appearance::{
    appearance::Appearance, resolve::resolve, system::SystemPrefs, typeface::Typeface,
};
use ds_style::appearance::{blur::BlurState, material::Material};
use ds_style::kit::UserStyle;
use ds_style::material::recipe::DEFAULT_TINT_ALPHA;
use ds_style::material::stack::MaterialStack;
use ds_style::scale::use_root_scale;
use ds_style::scope::{Scope, use_scope_provider};
use ds_style::space::{frame_vars::FrameVars, look::SpaceLook};
use ds_style::tokens::hex::Alpha;
use ds_style::tokens::{pixel::PixelToken, shape::Corner};
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
    #[props(default)] sheet: Option<&'static str>,
    #[props(default)] tint_alpha: Option<Alpha>,
    #[props(default)] chrome: Option<RootChrome>,
    #[props(default)] ground: Option<Ground>,
    #[props(default)] frame: Option<FrameTint>,
    #[props(default)] radius: Option<Corner>,
    #[props(default)] stack: Option<MaterialStack>,
    #[props(default)] scale: Option<Scale>,
    #[props(default)] window: WindowFrame,
    #[props(default)] extent: RootExtent,
    #[props(default)] typeface: Option<Typeface>,
    #[props(default)] user_style: ReadSignal<UserStyle>,
    #[props(default)] surface: Option<&'static str>,
    children: Element,
) -> Element {
    let typeface = typeface.unwrap_or(use_typeface());
    use_typeface_provider(typeface);
    let signals = use_hook(try_consume_context::<HostSignals>);
    let scale = use_root_scale(scale, signals.map(|signals| (signals.scale)()));
    let chrome = chrome.unwrap_or(RootChrome::of(material));
    let frame_tint = frame.unwrap_or(FrameTint::of(material, chrome));
    let ground = ground.unwrap_or(Ground::of(material));
    let resolved = resolve(appearance, look.theme, system);
    let mut element = use_hook(|| CopyValue::new(None::<Rc<MountedData>>));
    let click_root = use_context_provider(|| ClickRoot::of(element));
    let modality = signals.map_or(InputModality::default(), |signals| (signals.modality)());
    let activity = signals.map_or(Activity::Active, |signals| (signals.activity)());
    let env = use_scope_provider(Scope {
        resolved,
        scheme: resolved.scheme,
        material,
        blur,
        modality,
        activity,
    });
    let hover = use_hover_hub_provider(env);
    use_toast_hub_provider(env);
    use_context_provider(|| Signal::new(LayerStack::default()));
    use_overlays_provider();
    let frame = FrameVars::of(&look, resolved.scheme);
    let layers = use_frame_layers(&frame.gradient);
    let tint = tint_alpha.unwrap_or(DEFAULT_TINT_ALPHA);
    let corner = radius
        .map(crate::root::surface::radius_style)
        .unwrap_or_default();
    let stack = stack.map(|stack| stack.style_attr()).unwrap_or_default();
    let pixels = PixelToken::style_attr(scale);
    let style = format!(
        "{}{}--m-tint-alpha:{};{corner}{stack}{pixels}",
        frame.style_attr(),
        crate::root::surface::accent_text_style(material),
        tint.css()
    );
    let framing = window.attribute();
    let hover = match hover.warmth() {
        HoverWarmth::Warm => "warm",
        HoverWarmth::Cold => "cold",
    };
    rsx! {
        div {
            class: "ds",
            "data-theme": resolved.scheme.slug(),
            "data-typeface": typeface.slug(),
            "data-accent": resolved.accent.slug(),
            "data-motion": resolved.motion.slug(),
            "data-material": material.slug(),
            "data-blur": blur.slug(),
            "data-modality": modality.slug(),
            "data-activity": (activity == Activity::Inactive).then(|| activity.slug()),
            "data-hover": hover,
            "data-chrome": chrome.attribute(),
            "data-frame": frame_tint.attribute(),
            "data-ground": ground.attribute(),
            "data-corner": radius.and_then(Corner::attribute),
            "data-window-frame": framing,
            "data-extent": extent.attribute(),
            "data-surface": surface,
            style,
            onmounted: move |event: MountedEvent| element.set(Some(event.data())),
            // Last to hear a click: where the keyboard goes when it landed on nothing focusable.
            onclick: move |event: MouseEvent| click_root.clicked(&event),
            if stylesheet == Inject::Inline {
                style { {sheet.unwrap_or_else(crate::assembly::stylesheet::stylesheet)} }
            }
            // After the design-system sheet, so a rule of the person's wins by order.
            if !user_style.read().is_blank() {
                style { "data-ds-user": "", {user_style.read().0.clone()} }
            }
            match frame_tint {
                FrameTint::Opaque => frame_layers(layers),
                FrameTint::Tinted => rsx! {
                    div { class: "ds-frame", {frame_layers(layers)} }
                },
                FrameTint::None => rsx! {},
            }
            {framed(window, children)}
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
