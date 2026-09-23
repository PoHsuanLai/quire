//! The `--f-*` variables a `.ds` root carries inline, computed in Rust for the resolved scheme
//! (design/03-COLOR.md sections 4.5 and 8, design/21-SPACES.md section 2).
//!
//! Replaces mailo's `ui/paint.rs` `push_palette` and `grain_opacity`; the `-l`/`-d` copies
//! and `SYSTEM_ACCENT` are deleted.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::look::SpaceLook;
use crate::appearance::Scheme;

/// Every frame variable one Space paints in one scheme, as CSS values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVars {
    /// `--f-ink`: sidebar text.
    pub ink: String,
    /// `--f-ink-soft`: secondary sidebar text.
    pub ink_soft: String,
    /// `--f-ink-faint`: counts and meta.
    pub ink_faint: String,
    /// `--f-pill`: a selected pill.
    pub pill: String,
    /// `--f-pill-hover`: a row under the pointer.
    pub pill_hover: String,
    /// `--f-line`: hairlines on the frame.
    pub line: String,
    /// `--f-solid`: the first gradient stop, for surfaces that cannot paint the gradient.
    pub solid: String,
    /// `--f-grad`: the frame's `linear-gradient`.
    pub gradient: String,
    /// `--f-grain`: the grain tile's opacity, `grain / 100 x .20` light, `x .16` dark.
    pub grain_opacity: String,
    /// `--accent`, `--accent-soft`, `--accent-ink` when the card borrows the Space's hue.
    pub accent: Option<[String; 3]>,
}

impl FrameVars {
    /// The frame variables `look` paints in `scheme`.
    pub fn of(look: &SpaceLook, scheme: Scheme) -> Self {
        todo!()
    }

    /// The inline `style` attribute value: `--f-ink:#…;--f-ink-soft:#…;…`.
    pub fn style_attr(&self) -> String {
        todo!()
    }
}
