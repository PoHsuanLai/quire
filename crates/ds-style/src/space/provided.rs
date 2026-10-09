//! The Space a `Ds` root draws, readable by everything under it: the look, and the frame
//! variables it paints in the root's resolved scheme. An app that draws for itself (a canvas, a
//! texture) restyles from these when the person switches Space or edits its look.

use super::frame_vars::FrameVars;
use super::look::SpaceLook;
use crate::appearance::theme::Scheme;
use crate::scope::Scope;
use dioxus::prelude::*;

/// Provide `look` to the subtree, updating the provided value when it changes. Nothing here
/// reads the signal, so the write does not re-render the caller.
pub fn use_space_look_provider(look: &SpaceLook) -> Signal<SpaceLook> {
    let mut provided = use_context_provider(|| Signal::new(look.clone()));
    if *provided.peek() != *look {
        provided.set(look.clone());
    }
    provided
}

/// The look of the Space the enclosing `Ds` draws, redrawn when it changes; the default look
/// outside a `Ds`.
pub fn use_space_look() -> SpaceLook {
    try_use_context::<Signal<SpaceLook>>().map_or_else(SpaceLook::default, |look| look())
}

/// The frame variables that look paints in the enclosing root's scheme, redrawn when either
/// changes; the light scheme's outside a `Ds`.
pub fn use_space_frame() -> FrameVars {
    let scheme = try_use_context::<Signal<Scope>>().map_or(Scheme::Light, |scope| scope().scheme);
    FrameVars::of(&use_space_look(), scheme)
}
