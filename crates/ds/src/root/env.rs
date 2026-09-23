//! What every component under a `Ds` can read: the resolved appearance, the material and the
//! blur state of the scope it is in.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::appearance::{Resolved, Scheme};
use crate::material::{BlurState, Material};
use dioxus::prelude::*;

/// How the person last drove the surface: `data-modality` on `.ds`.
///
/// blitz-dom hard-codes `:focus-visible` and `:focus-within` to false and a click does not focus
/// a button (spike S12), so the focus ring is `.ds[*|data-modality=keyboard] :focus` and the host
/// says which modality is current.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InputModality {
    /// The last input was a pointer: no focus rings.
    #[default]
    Pointer,
    /// The last input was a key: focus rings show.
    Keyboard,
}

impl InputModality {
    /// The `data-modality` value.
    pub fn slug(self) -> &'static str {
        match self {
            InputModality::Pointer => "pointer",
            InputModality::Keyboard => "keyboard",
        }
    }
}

/// The host's modality, provided as root context by `ds-native` (which sees raw input); `Ds`
/// stamps it on `.ds`. Without one, `Ds` stamps `pointer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostModality(pub Signal<InputModality>);

/// The enclosing scope, as `Ds` and `Surface` provide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Env {
    /// The root's resolved scheme, accent and motion level.
    pub resolved: Resolved,
    /// The scheme of the nearest `Surface` that overrides it, else the root's.
    pub scheme: Scheme,
    /// The nearest material.
    pub material: Material,
    /// Whether the compositor blurs behind this surface.
    pub blur: BlurState,
    /// How the person last drove the surface.
    pub modality: InputModality,
}

/// The enclosing scope. Panics outside a `Ds`: every quire component is drawn inside one.
pub fn use_env() -> Env {
    todo!()
}
