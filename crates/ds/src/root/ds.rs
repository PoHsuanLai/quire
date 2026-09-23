//! The root: `div.ds` carrying `data-theme`, `data-accent`, `data-motion`, `data-material`,
//! `data-blur` and `data-modality`, with the frame's `--f-*` inline; then the stylesheet (when
//! inlined), the frame layers and grain on a Window, the children, and the overlay and toast
//! hosts. It provides `Env`, `HoverHub`, `ToastHub`, `LayerStack` and `Overlays` as context.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::appearance::{Appearance, SystemPrefs};
use crate::material::{BlurState, Material};
use crate::space::SpaceLook;
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
    todo!()
}
